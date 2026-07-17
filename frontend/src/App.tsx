import AuthPage from "./pages/AuthPage";
import HomePage from "./pages/HomePage";

function App() {
  const path = window.location.pathname;

  if (path === "/login") {
    return <AuthPage mode="login" />;
  }

  if (path === "/signup") {
    return <AuthPage mode="signup" />;
  }

  return <HomePage />;
}

export default App;
