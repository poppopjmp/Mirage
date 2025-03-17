FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files and install dependencies
COPY services/web-ui/package.json services/web-ui/package-lock.json ./
RUN npm ci

# Copy source code
COPY services/web-ui .

# Build the application
RUN npm run build

# Use Nginx to serve the application
FROM nginx:alpine

# Copy built assets from builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Copy custom nginx config if needed
COPY docker/services/web-ui/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
