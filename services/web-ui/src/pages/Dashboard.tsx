import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Grid,
  Typography,
  Paper,
  Button,
  CircularProgress,
  Divider,
  useTheme,
  LinearProgress
} from '@mui/material';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell
} from 'recharts';
import { api } from '../services/api';
import { useSnackbar } from '../contexts/SnackbarContext';

// Sample data for charts
const lineChartData = [
  { name: 'Jan', domains: 4, ips: 5, emails: 8 },
  { name: 'Feb', domains: 7, ips: 9, emails: 10 },
  { name: 'Mar', domains: 10, ips: 12, emails: 14 },
  { name: 'Apr', domains: 14, ips: 15, emails: 12 },
  { name: 'May', domains: 18, ips: 14, emails: 9 },
  { name: 'Jun', domains: 15, ips: 13, emails: 7 },
];

const barChartData = [
  { name: 'Subdomains', count: 120 },
  { name: 'IP Addresses', count: 87 },
  { name: 'Email Addresses', count: 53 },
  { name: 'URLs', count: 142 },
  { name: 'Companies', count: 36 },
  { name: 'People', count: 22 },
];

const pieChartData = [
  { name: 'Active', value: 70 },
  { name: 'Completed', value: 15 },
  { name: 'Failed', value: 5 },
  { name: 'Scheduled', value: 10 },
];

const COLORS = ['#7e57c2', '#26a69a', '#ef5350', '#66bb6a'];

interface Stat {
  title: string;
  value: number;
  unit?: string;
  change?: number;
}

interface RecentScan {
  id: string;
  target: string;
  status: 'active' | 'completed' | 'failed' | 'scheduled';
  createdAt: string;
  progress?: number;
}

const Dashboard: React.FC = () => {
  const [stats, setStats] = useState<Stat[]>([
    { title: 'Total Scans', value: 156, change: 12 },
    { title: 'Entities Collected', value: 24863, change: 1834 },
    { title: 'Active Modules', value: 34, unit: 'modules', change: 2 },
    { title: 'Scan Queue', value: 3, unit: 'scans', change: -1 },
  ]);
  
  const [recentScans, setRecentScans] = useState<RecentScan[]>([
    { id: '1', target: 'example.com', status: 'active', createdAt: '2023-06-10T14:30:00Z', progress: 67 },
    { id: '2', target: 'test-corp.org', status: 'completed', createdAt: '2023-06-09T09:15:00Z' },
    { id: '3', target: 'securityresearch.dev', status: 'scheduled', createdAt: '2023-06-10T18:00:00Z' },
    { id: '4', target: 'vuln-tracker.io', status: 'failed', createdAt: '2023-06-08T11:45:00Z' },
  ]);
  
  const [isLoading, setIsLoading] = useState(false);
  const { showSnackbar } = useSnackbar();
  const theme = useTheme();
  
  useEffect(() => {
    // In a real implementation, this would fetch data from the API
    const fetchDashboardData = async () => {
      setIsLoading(true);
      try {
        // Uncomment this in a real implementation
        /*
        const [statsResponse, scansResponse] = await Promise.all([
          api.get('/api/v1/dashboard/stats'),
          api.get('/api/v1/scans/recent')
        ]);
        
        setStats(statsResponse.data);
        setRecentScans(scansResponse.data);
        */
        
        // Simulate API delay for demo
        await new Promise(resolve => setTimeout(resolve, 1000));
        
      } catch (error) {
        console.error('Failed to fetch dashboard data:', error);
        showSnackbar('Failed to load dashboard data. Please try again.', 'error');
      } finally {
        setIsLoading(false);
      }
    };
    
    fetchDashboardData();
  }, [showSnackbar]);
  
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };
  
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return '#7e57c2';
      case 'completed': return '#66bb6a';
      case 'failed': return '#ef5350';
      case 'scheduled': return '#26a69a';
      default: return '#aaaaaa';
    }
  };
  
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>
      
      {isLoading ? (
        <Box display="flex" justifyContent="center" my={8}>
          <CircularProgress size={60} />
        </Box>
      ) : (
        <>
          {/* Key Stats */}
          <Grid container spacing={3} mb={4}>
            {stats.map((stat, index) => (
              <Grid item xs={12} sm={6} md={3} key={index}>
                <Paper elevation={2} sx={{ p: 3, height: '100%' }}>
                  <Typography variant="subtitle2" color="text.secondary">
                    {stat.title}
                  </Typography>
                  <Typography variant="h4" component="div" mt={1}>
                    {stat.value.toLocaleString()}
                    {stat.unit && <Typography variant="body2" component="span" ml={1}>{stat.unit}</Typography>}
                  </Typography>
                  {stat.change !== undefined && (
                    <Typography 
                      variant="body2" 
                      color={stat.change >= 0 ? 'success.main' : 'error.main'}
                      mt={1}
                    >
                      {stat.change >= 0 ? '+' : ''}{stat.change} from last week
                    </Typography>
                  )}
                </Paper>
              </Grid>
            ))}
          </Grid>
          
          {/* Charts */}
          <Grid container spacing={3} mb={4}>
            {/* Entity Discovery Over Time */}
            <Grid item xs={12} md={8}>
              <Paper elevation={2} sx={{ p: 3, height: '100%' }}>
                <Typography variant="h6" gutterBottom>
                  Entity Discovery Over Time
                </Typography>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={lineChartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="domains" stroke="#7e57c2" activeDot={{ r: 8 }} />
                    <Line type="monotone" dataKey="ips" stroke="#26a69a" />
                    <Line type="monotone" dataKey="emails" stroke="#ef5350" />
                  </LineChart>
                </ResponsiveContainer>
              </Paper>
            </Grid>
            
            {/* Scan Status Distribution */}
            <Grid item xs={12} md={4}>
              <Paper elevation={2} sx={{ p: 3, height: '100%' }}>
                <Typography variant="h6" gutterBottom>
                  Scan Status Distribution
                </Typography>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={pieChartData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="value"
                      label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                    >
                      {pieChartData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value) => [`${value} scans`, 'Count']} />
                  </PieChart>
                </ResponsiveContainer>
              </Paper>
            </Grid>
            
            {/* Entity Type Distribution */}
            <Grid item xs={12}>
              <Paper elevation={2} sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Entity Type Distribution
                </Typography>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={barChartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="count" fill="#7e57c2" />
                  </BarChart>
                </ResponsiveContainer>
              </Paper>
            </Grid>
          </Grid>
          
          {/* Recent Scans */}
          <Paper elevation={2} sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Recent Scans
            </Typography>
            <Grid container spacing={2}>
              {recentScans.map((scan) => (
                <Grid item xs={12} md={6} lg={3} key={scan.id}>
                  <Card variant="outlined">
                    <CardContent>
                      <Typography variant="h6" noWrap sx={{ mb: 1 }}>
                        {scan.target}
                      </Typography>
                      <Box display="flex" justifyContent="space-between" alignItems="center" mb={1}>
                        <Typography variant="body2" color="text.secondary">
                          {formatDate(scan.createdAt)}
                        </Typography>
                        <Typography 
                          variant="body2" 
                          sx={{ 
                            px: 1, 
                            py: 0.5, 
                            borderRadius: 1, 
                            bgcolor: `${getStatusColor(scan.status)}20`,
                            color: getStatusColor(scan.status),
                            fontWeight: 'medium'
                          }}
                        >
                          {scan.status.charAt(0).toUpperCase() + scan.status.slice(1)}
                        </Typography>
                      </Box>
                      {scan.progress !== undefined && (
                        <Box sx={{ display: 'flex', alignItems: 'center' }}>
                          <Box sx={{ width: '100%', mr: 1 }}>
                            <LinearProgress 
                              variant="determinate" 
                              value={scan.progress} 
                              sx={{ 
                                height: 8, 
                                borderRadius: 4,
                                bgcolor: theme.palette.background.default,
                                '& .MuiLinearProgress-bar': {
                                  bgcolor: getStatusColor(scan.status),
                                }
                              }} 
                            />
                          </Box>
                          <Box sx={{ minWidth: 35 }}>
                            <Typography variant="body2" color="text.secondary">{`${Math.round(scan.progress)}%`}</Typography>
                          </Box>
                        </Box>
                      )}
                      <Button 
                        variant="outlined" 
                        size="small"
                        sx={{ mt: 2 }}
                        onClick={() => navigate(`/scans/${scan.id}`)}
                      >
                        View Details
                      </Button>
                    </CardContent>
                  </Card>
                </Grid>
              ))}
            </Grid>
            <Box display="flex" justifyContent="center" mt={3}>
              <Button variant="contained" onClick={() => navigate('/scans')}>
                View All Scans
              </Button>
            </Box>
          </Paper>
        </>
      )}
    </Box>
  );
};

export default Dashboard;
