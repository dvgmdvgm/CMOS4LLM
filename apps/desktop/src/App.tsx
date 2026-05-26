import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [version, setVersion] = useState("");

  useEffect(() => {
    invoke<string>("get_version").then(setVersion);
  }, []);

  return (
    <div className="container">
      <h1>CMOS Cognitive Console</h1>
      <p className="version">{version || "connecting..."}</p>
      <p className="status">Daemon: ready</p>
    </div>
  );
}

export default App;
