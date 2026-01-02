import { useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { AppLayout } from "./components/layout/AppLayout"

export default function App() {
  useEffect(() => {
    // Test Tauri connection
    invoke("plugin:cadhy-bridge|project_ping")
      .then((result) => console.log("[App] Tauri connected:", result))
      .catch((err) => console.error("[App] Tauri error:", err))
  }, [])

  return <AppLayout />
}
