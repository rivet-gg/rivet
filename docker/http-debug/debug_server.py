from flask import Flask, request, jsonify
import gzip
import zlib
from datetime import datetime

app = Flask(__name__)

def log_request_details():
    """Log all request details in a formatted way"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    print("=" * 80)
    print(f"[{timestamp}] NEW REQUEST")
    print("=" * 80)
    
    # Method and URL
    print(f"Method: {request.method}")
    print(f"URL: {request.url}")
    print(f"Path: {request.path}")
    print(f"Query String: {request.query_string.decode('utf-8')}")
    
    # Headers
    print("\n--- HEADERS ---")
    for header_name, header_value in request.headers:
        print(f"{header_name}: {header_value}")
    
    # Query Parameters
    if request.args:
        print("\n--- QUERY PARAMETERS ---")
        for key, value in request.args.items():
            print(f"{key}: {value}")
    
    # Form Data
    if request.form:
        print("\n--- FORM DATA ---")
        for key, value in request.form.items():
            print(f"{key}: {value}")
    
    # Files
    if request.files:
        print("\n--- FILES ---")
        for key, file in request.files.items():
            print(f"{key}: {file.filename} (Content-Type: {file.content_type})")
    
    # Raw Body with decompression support
    try:
        raw_body = request.get_data()
        content_encoding = request.headers.get('Content-Encoding', '').lower()
        
        print("\n--- REQUEST BODY ---")
        
        if not raw_body:
            print("(empty)")
        else:
            # Handle compressed content
            decompressed_body = None
            
            if content_encoding == 'gzip':
                try:
                    decompressed_body = gzip.decompress(raw_body).decode('utf-8')
                    print("(Content was gzip-compressed, showing decompressed version)")
                except Exception as e:
                    print(f"Failed to decompress gzip content: {e}")
            elif content_encoding == 'deflate':
                try:
                    decompressed_body = zlib.decompress(raw_body).decode('utf-8')
                    print("(Content was deflate-compressed, showing decompressed version)")
                except Exception as e:
                    print(f"Failed to decompress deflate content: {e}")
            elif content_encoding in ['br', 'brotli']:
                print("(Content is brotli-compressed - brotli decompression not available)")
                print("Raw compressed data (first 200 bytes):")
                print(repr(raw_body[:200]))
            elif content_encoding == 'zstd':
                print("(Content is zstd-compressed - zstd decompression not available)")
                print("Raw compressed data (first 200 bytes):")
                print(repr(raw_body[:200]))
            else:
                # No compression or unknown compression
                try:
                    decompressed_body = raw_body.decode('utf-8')
                except UnicodeDecodeError:
                    print("(Binary content - showing first 200 bytes as hex)")
                    print(raw_body[:200].hex())
            
            # Display the decompressed content
            if decompressed_body:
                print(decompressed_body)
                
                # Also show raw length info
                print(f"\nRaw body length: {len(raw_body)} bytes")
                print(f"Decompressed length: {len(decompressed_body)} bytes")
            
    except Exception as e:
        print(f"\n--- REQUEST BODY ---")
        print(f"Error reading body: {e}")
    
    print("=" * 80)
    print()

# Catch all routes for any HTTP method
@app.route('/', defaults={'path': ''}, methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'])
@app.route('/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'])
def debug_endpoint(path):
    log_request_details()
    
    # Return a simple response
    response_data = {
        "message": "Request received and logged",
        "method": request.method,
        "path": f"/{path}" if path else "/",
        "timestamp": datetime.now().isoformat()
    }
    
    return jsonify(response_data), 200

if __name__ == '__main__':
    print("Starting HTTP Debug Server...")
    print("All incoming requests will be logged to console")
    print("Server listening on port 8080")
    print("=" * 80)
    app.run(host='0.0.0.0', port=8080, debug=False)