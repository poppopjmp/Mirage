FROM node:18-slim as builder

WORKDIR /app

# Copy package files and install dependencies
COPY ./web-ui/package*.json ./
RUN npm ci

# Copy the source code
COPY ./services/web-ui ./

# Build the application
RUN npm run build

# Use Nginx for serving the application
FROM nginx:alpine

# Copy the built app
COPY --from=builder /app/build /usr/share/nginx/html

# Copy the Nginx configuration
COPY ./docker/services/web-ui/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
