# Getting Started with Mirage

This guide will help you get started using Mirage for your threat intelligence and security operations needs.

## Installation

### Cloud Deployment

If you're using Mirage as a managed service, simply:

1. Create an account at [mirage-platform.example.com](https://mirage-platform.example.com)
2. Log in to your account
3. Follow the onboarding wizard to set up your organization

### On-Premise Deployment

For on-premise deployment:

1. Download the installer from the [releases page](https://github.com/your-org/mirage/releases)
2. Follow the installation guide for your platform:
   - [Linux Installation](guides/installation-linux.md)
   - [Windows Installation](guides/installation-windows.md)
   - [Kubernetes Installation](guides/installation-kubernetes.md)

## Initial Configuration

1. **Create an Admin Account**
   
   Upon first login, you'll be prompted to create an admin account.

2. **Add Users and Teams**
   
   Navigate to the User Management section to add users and organize them into teams.

3. **Configure Data Sources**
   
   Set up your initial data sources in the Data Collection section.

4. **Create Your First Scan**
   
   Navigate to the Scan Orchestration page and create your first intelligence gathering scan.

## Core Concepts

### Scans

A scan is a configured intelligence gathering operation that collects data from various sources according to defined parameters.

### Modules

Modules are pluggable components that extend Mirage's capabilities. They include:
- Data Collection Modules
- Enrichment Modules
- Analysis Modules
- Visualization Modules

### Reports

Reports are customizable outputs that present the collected and analyzed data in a structured format.

## Basic Usage

### Running a Scan

1. Navigate to the Scan Dashboard
2. Click "New Scan"
3. Select the target and scope
4. Choose relevant modules
5. Configure scan parameters
6. Click "Start Scan"

### Viewing Results

1. Navigate to the Visualization section
2. Select your scan from the list
3. Explore the data using the interactive visualization tools
4. Apply filters to focus on specific data points

### Generating Reports

1. Navigate to the Reports section
2. Click "New Report"
3. Select report template
4. Choose data sources
5. Generate and download the report

## Next Steps

- Explore [Advanced Configuration](guides/advanced-configuration.md)
- Learn about [Custom Modules](guides/custom-modules.md)
- Set up [Integrations](guides/integrations.md) with your existing tools
