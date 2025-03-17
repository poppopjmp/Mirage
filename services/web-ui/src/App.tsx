import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ThemeProvider, CssBaseline } from '@mui/material';
import { theme } from './theme';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import ScanList from './pages/scans/ScanList';
import ScanDetail from './pages/scans/ScanDetail';
import CreateScan from './pages/scans/CreateScan';
import ModuleList from './pages/modules/ModuleList';
import DataExplorer from './pages/data/DataExplorer';
import VisualizationList from './pages/visualizations/VisualizationList';
import VisualizationDetail from './pages/visualizations/VisualizationDetail';
import ReportList from './pages/reports/ReportList';
import Login from './pages/auth/Login';
import Register from './pages/auth/Register';
import UserProfile from './pages/users/UserProfile';
import NotFound from './pages/NotFound';
import ProtectedRoute from './components/ProtectedRoute';
import { AuthProvider } from './contexts/AuthContext';
import { SnackbarProvider } from './contexts/SnackbarContext';

function App() {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <SnackbarProvider>
        <AuthProvider>
          <Router>
            <Routes>
              <Route path="/login" element={<Login />} />
              <Route path="/register" element={<Register />} />
              <Route
                path="/"
                element={
                  <ProtectedRoute>
                    <Layout />
                  </ProtectedRoute>
                }
              >
                <Route index element={<Dashboard />} />
                <Route path="scans">
                  <Route index element={<ScanList />} />
                  <Route path="new" element={<CreateScan />} />
                  <Route path=":scanId" element={<ScanDetail />} />
                </Route>
                <Route path="modules" element={<ModuleList />} />
                <Route path="data" element={<DataExplorer />} />
                <Route path="visualizations">
                  <Route index element={<VisualizationList />} />
                  <Route path=":visualizationId" element={<VisualizationDetail />} />
                </Route>
                <Route path="reports" element={<ReportList />} />
                <Route path="profile" element={<UserProfile />} />
                <Route path="*" element={<NotFound />} />
              </Route>
            </Routes>
          </Router>
        </AuthProvider>
      </SnackbarProvider>
    </ThemeProvider>
  );
}

export default App;
