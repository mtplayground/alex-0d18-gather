import { AuthProvider } from "./auth/AuthProvider";
import { ProtectedRoute } from "./auth/ProtectedRoute";
import AuthPage from "./pages/AuthPage";
import DashboardPage from "./pages/DashboardPage";
import EventCreatePage from "./pages/EventCreatePage";
import HomePage from "./pages/HomePage";
import ProfilePage from "./pages/ProfilePage";

function App() {
  return (
    <AuthProvider>
      <AppRoutes />
    </AuthProvider>
  );
}

function AppRoutes() {
  const path = window.location.pathname;

  if (path === "/login") {
    return <AuthPage mode="login" />;
  }

  if (path === "/signup") {
    return <AuthPage mode="signup" />;
  }

  if (path === "/dashboard") {
    return (
      <ProtectedRoute>
        <DashboardPage />
      </ProtectedRoute>
    );
  }

  if (path === "/profile") {
    return (
      <ProtectedRoute>
        <ProfilePage />
      </ProtectedRoute>
    );
  }

  if (path === "/events/new") {
    return (
      <ProtectedRoute>
        <EventCreatePage />
      </ProtectedRoute>
    );
  }

  return <HomePage />;
}

export default App;
