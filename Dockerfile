# Use an official Rust runtime as a parent image
FROM rust:latest

# Set the working directory to /app
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY . /app

# Install any needed dependencies
RUN cargo install --path .

# Make port 80 available to the world outside this container
EXPOSE 80

# Define environment variable
ENV DATABASE_URL postgres://myuser:mypassword@db:5432/mydb

# Run your application when the container launches
CMD ["cargo", "run"]
