use super::Transform;

#[derive(Clone, Copy, Debug)]
pub struct SpatialConfig {
	/// Within this distance the source plays at full volume.
	pub min_distance: f32,
	/// Beyond this distance the source is inaudible.
	pub max_distance: f32,
}

impl Default for SpatialConfig {
	fn default() -> Self {
		Self {
			min_distance: 1.0,
			max_distance: 25.0,
		}
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpatialOutput {
	pub left: f32,
	pub right: f32,
}

/// Computes stereo gains for a source at `source_position`, relative to the
/// listener's position and orientation.
///
/// Volume falls off linearly between `min_distance` and `max_distance`, and
/// panning uses constant-power panning based on the source's horizontal angle
/// in the listener's local space.
pub fn spatialize(
	listener: Transform,
	source_position: [f32; 3],
	config: &SpatialConfig,
) -> SpatialOutput {
	let relative = [
		source_position[0] - listener.position[0],
		source_position[1] - listener.position[1],
		source_position[2] - listener.position[2],
	];

	let distance = length(relative);
	if distance >= config.max_distance {
		return SpatialOutput::default();
	}

	let local_right = project_onto_right(listener.rotation, relative);

	let pan = if distance > f32::EPSILON {
		(local_right / distance).clamp(-1.0, 1.0)
	} else {
		0.0
	};

	let attenuation = if distance <= config.min_distance {
		1.0
	} else {
		((config.max_distance - distance) / (config.max_distance - config.min_distance))
			.clamp(0.0, 1.0)
	};

	// Map pan [-1, 1] to an angle [0, PI/2] and use sin/cos so the total
	// power stays constant as the source moves around the listener.
	let angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;

	SpatialOutput {
		left: attenuation * angle.cos(),
		right: attenuation * angle.sin(),
	}
}

/// Projects a world-space vector onto the listener's right axis, which is the
/// component that determines stereo panning.
///
/// The listener rotation is applied as `world = Ry(yaw) * Rx(pitch) * Rz(roll)`,
/// so the right axis is the first column of that rotation matrix.
fn project_onto_right(rotation: [f32; 3], vector: [f32; 3]) -> f32 {
	let [pitch, yaw, roll] = rotation;
	let (sin_pitch, cos_pitch) = pitch.sin_cos();
	let (sin_yaw, cos_yaw) = yaw.sin_cos();
	let (sin_roll, cos_roll) = roll.sin_cos();

	let right = [
		(sin_yaw * sin_pitch).mul_add(sin_roll, cos_yaw * cos_roll),
		cos_pitch * sin_roll,
		(cos_yaw * sin_pitch).mul_add(sin_roll, -sin_yaw * cos_roll),
	];

	dot(right, vector)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
	a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn length(vector: [f32; 3]) -> f32 {
	dot(vector, vector).sqrt()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn listener_at_origin() -> Transform {
		Transform::default()
	}

	#[test]
	fn source_ahead_is_centered() {
		let output = spatialize(
			listener_at_origin(),
			[0.0, 0.0, -2.0],
			&SpatialConfig::default(),
		);
		assert!((output.left - output.right).abs() < 1e-5);
		assert!(output.left > 0.0);
	}

	#[test]
	fn source_to_the_right_pans_right() {
		let output = spatialize(
			listener_at_origin(),
			[2.0, 0.0, 0.0],
			&SpatialConfig::default(),
		);
		assert!(output.right > output.left);
	}

	#[test]
	fn source_to_the_left_pans_left() {
		let output = spatialize(
			listener_at_origin(),
			[-2.0, 0.0, 0.0],
			&SpatialConfig::default(),
		);
		assert!(output.left > output.right);
	}

	#[test]
	fn yaw_rotates_panning() {
		// Listener turned 90 degrees to the right (yaw = -PI/2): a source that
		// was directly to the right is now directly ahead, so it should center.
		let listener = Transform {
			position: [0.0, 0.0, 0.0],
			rotation: [0.0, -std::f32::consts::FRAC_PI_2, 0.0],
		};
		let output = spatialize(listener, [2.0, 0.0, 0.0], &SpatialConfig::default());
		assert!((output.left - output.right).abs() < 1e-4);
	}

	#[test]
	fn volume_decreases_with_distance() {
		let config = SpatialConfig::default();
		let near = spatialize(listener_at_origin(), [0.0, 0.0, -2.0], &config);
		let far = spatialize(listener_at_origin(), [0.0, 0.0, -10.0], &config);
		assert!(near.left > far.left);
	}

	#[test]
	fn silent_beyond_max_distance() {
		let config = SpatialConfig::default();
		let output = spatialize(
			listener_at_origin(),
			[0.0, 0.0, -(config.max_distance + 1.0)],
			&config,
		);
		assert_eq!(output, SpatialOutput::default());
	}

	#[test]
	fn full_volume_within_min_distance() {
		let config = SpatialConfig::default();
		let output = spatialize(listener_at_origin(), [0.0, 0.0, -0.5], &config);
		assert!((output.left - output.right).abs() < 1e-6);
		// Constant-power panning keeps total power at full volume.
		let power = output
			.right
			.mul_add(output.right, output.left * output.left);
		assert!((power - 1.0).abs() < 1e-4);
	}
}
