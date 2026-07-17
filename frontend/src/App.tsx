import { AuthProvider } from "./auth/AuthProvider";
import { ProtectedRoute } from "./auth/ProtectedRoute";
import AuthPage from "./pages/AuthPage";
import DashboardPage from "./pages/DashboardPage";
import EventCreatePage from "./pages/EventCreatePage";
import EventDetailPage from "./pages/EventDetailPage";
import HomePage from "./pages/HomePage";
import InvitationRsvpPage from "./pages/InvitationRsvpPage";
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

  const eventDetailMatch = path.match(/^\/events\/([^/]+)$/);
  if (eventDetailMatch) {
    return (
      <ProtectedRoute>
        <EventDetailPage eventId={decodeURIComponent(eventDetailMatch[1])} />
      </ProtectedRoute>
    );
  }

  const invitationMatch = path.match(/^\/invitations\/([^/]+)$/);
  if (invitationMatch) {
    return (
      <ProtectedRoute>
        <InvitationRsvpPage shareToken={decodeURIComponent(invitationMatch[1])} />
      </ProtectedRoute>
    );
  }

  return <HomePage />;
}

export default App;
