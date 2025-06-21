FROM python:3.11-slim

WORKDIR /app

# Install required packages
RUN pip install flask

# Create the debug server
COPY debug_server.py .

# Expose port 8080
EXPOSE 8080

# Run the server
CMD ["python", "debug_server.py"]