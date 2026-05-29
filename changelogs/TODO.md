# TODOS:

- Keep better track of loading states.
- Figure out why stuff gets stuck forever. Like if one thing is downloading, can't open a folder?
- Make sure things work offline as much as possible. Especially stuff from remotemod.
- in database:
  - check each mod to see if zip structure is correct.
  - make sure one failure doesn't break the whole thing. it should preserve the previous mod if so.
- Make sure manifests don't get installed if anything in the process fails. It seemed to fail when a dependency of a dependency failed.

# Done:

- Add Z: to UE4SS paths. Need to figure out how to handle that generically.
- Check all new token types being replaced.
- Hide runnable self run button on linux.
- the bepinex config thing should be a dependency by itself instead of the entire bepinex ugh
- are dependencies even working? uevr didnt seem to download it.
- Show download progress somewhere.
- Handle legacy bepinex.cfg in database
- The "operating system" thing in mod dependencies should be in the dependency itself. if dependency is wine-only, install only for wine. could be that mods can have gameOs and currentOs (think of better name maybe) fields that both need to be true to mean "wine".
- Actually call wine dll override setters.
- Check if the way winedlloverrides is being set in steam_proton.rs needs to be that way. does it need to be normalized like that? i'd like to be able to set it in the action config like a keyvalue map.
- make sure local/remote merging is correct.
- dependecy should only be installed if install is defined, otherwise only download locally.
- frontend no longer doing some stuff on mod install, like updating, downloading remote config. Check that.
- need to rethink the entire "local mod" thing. maybe it should be seen more as a cache.
- Prevent UE4SS from downloading when running UEVR
- Maybe new layout for game modal?
- ignore incompatible depenedncies.

Actions that a mod (or loader) can have:

- Install mod locally (all mods need this one I guess).
  - Implies uninstall action.
  - Implies "open local mod folder".
- Run mod executable by itself.
- Run mod executable with game.
- Install mod on game.
  - Implies uninstall action.
  - Implies optional "open installed mod folder".
  - Config needs:
    - Override folder name.
    - Which files/folders to copy from zip, and where to copy them to (use tokens here).
    - Files to write from scratch. Use tokens on file contents and target path.
      - For BepInEx we would write doorstop_config.ini and BepInEx.cfg.
      - For UE4SS we would write overrides.txt.
    - Get wine dll overrides
- Get mod config path for game.
