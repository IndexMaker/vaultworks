#!python3

import sys
import ast
import json
from decimal import Decimal, getcontext

# Set precision high enough to handle 128-bit numbers + 18 decimals safely
getcontext().prec = 50

def parse_fixed_128(byte_array, decimals=18):
    """
    Parses a byte array into 128-bit fixed-point values (18 decimals).
    Treats the byte array as a raw packed sequence of uint128s.
    """
    step = 16  # 128 bits = 16 bytes
    results = []
    
    divisor = Decimal(10**decimals)

    for i in range(0, len(byte_array), step):
        chunk = byte_array[i:i + step]
        if len(chunk) < step:
            break

        # Reconstruct integer from Little-Endian bytes as per spec requirement
        val = int.from_bytes(chunk, byteorder='little')

        # Convert to Decimal for high-precision division
        fixed_val = Decimal(val) / divisor
        
        # We use normalize() to keep the output clean, 
        # but cast to float for the JSON representation
        results.append(float(fixed_val.normalize()))

    return results

def handle_hex_input(hex_str):
    """
    Parses raw hex string directly into bytes.
    Does not assume ABI dynamic headers (offset/length).
    """
    hex_str = hex_str.strip().lower()
    if hex_str.startswith('0x'):
        hex_str = hex_str[2:]
    
    try:
        # Convert hex string directly to raw bytes
        raw_bytes = bytes.fromhex(hex_str)
        return list(raw_bytes)
    except ValueError:
        return None

def main():
    input_str = ""

    if len(sys.argv) > 1:
        input_str = sys.argv[1]
    elif not sys.stdin.isatty():
        input_str = sys.stdin.read().strip()

    if not input_str:
        sys.exit(0)

    try:
        # Determine if input is hex or a Python-style list string
        is_hex = input_str.startswith('0x') or all(c in '0123456789abcdefABCDEFx' for c in input_str.strip()[:10])
        
        if is_hex:
            byte_list = handle_hex_input(input_str)
        else:
            cleaned_input = input_str.strip()
            if not cleaned_input.startswith('['):
                cleaned_input = f"[{cleaned_input}]"
            byte_list = ast.literal_eval(cleaned_input)
        
        if byte_list is None:
            sys.exit(1)

        values = parse_fixed_128(byte_list)

        # Output the vector as a JSON array
        print(json.dumps(values))
            
    except Exception:
        sys.exit(1)

if __name__ == "__main__":
    main()

