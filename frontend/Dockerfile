# Use an official Node.js runtime as a parent image with a specific version
FROM node:20-alpine

# Set the working directory in the container
WORKDIR /app

# Install app dependencies by copying package.json and package-lock.json first
COPY package*.json ./

RUN npm install

# Copy the rest of the application code
COPY . .

# Build the app for production
RUN npm run build

# Expose port 3000 for the server
EXPOSE 3000

# Use serve to run the app
RUN npm install -g serve
CMD ["serve", "-s", "build"]

# # Use the official Node.js runtime as the base image
# FROM node:20 as build

# # Set the working directory in the container
# WORKDIR /app

# # Copy package.json and package-lock.json to the working directory
# COPY package*.json ./

# # Install dependencies
# RUN npm install

# # Copy the entire application code to the container
# COPY . .

# # Build the React app for production
# RUN npm run build

# # Use Nginx as the production server
# FROM nginx:alpine

# # Copy the built React app to Nginx's web server directory
# COPY --from=build /app/build /usr/share/nginx/html

# # Expose port 80 for the Nginx server
# EXPOSE 80

# # Start Nginx when the container runs
# CMD ["nginx", "-g", "daemon off;"]