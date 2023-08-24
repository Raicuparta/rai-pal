import {
  Container,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
} from "@chakra-ui/react";
import { HomePage } from "./pages/home-page";

const pages = [
  { title: "Home", component: HomePage },
  { title: "Find Games", component: HomePage },
  { title: "Settings", component: HomePage },
];

function App() {
  return (
    <Tabs isFitted display="flex" flexDirection={"column"} height="100vh">
      <TabList>
        <Container maxWidth="2xl" display="flex" flexDirection="row">
          {pages.map((page) => (
            <Tab>{page.title}</Tab>
          ))}
        </Container>
      </TabList>
      <TabPanels flex="1" overflow={"auto"}>
        {pages.map((page) => (
          <Container maxWidth="2xl">
            <TabPanel>{<page.component />}</TabPanel>
          </Container>
        ))}
      </TabPanels>
    </Tabs>
  );
}

export default App;
