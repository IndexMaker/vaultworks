#!python3

import sys
import json
from decimal import Decimal, getcontext

# Maintain 50-digit precision to handle 128-bit scaling safely
getcontext().prec = 50

def encode_fixed_128(values, decimals=18):
    """
    Converts a list of Decimal values into a packed byte array.
    """
    multiplier = Decimal(10**decimals)
    packed_bytes = bytearray()

    for val in values:
        # Scale the Decimal value and convert to integer
        # This handles the 18-decimal fixed-point requirement
        scaled_val = int(val * multiplier)
        
        try:
            # 128-bit (16 bytes), Little-Endian
            byte_chunk = scaled_val.to_bytes(16, byteorder='little')
            packed_bytes.extend(byte_chunk)
        except OverflowError:
            raise ValueError(f"Value {val} is too large for 128-bit representation.")

    return packed_bytes

def main():
    input_str = ""

    if len(sys.argv) > 1:
        input_str = sys.argv[1]
    elif not sys.stdin.isatty():
        input_str = sys.stdin.read().strip()

    if not input_str:
        print("Usage: python vector_to_bytes.py '[1.0, 0.005, 0.1]'")
        sys.exit(0)

    try:
        # We use json.loads with parse_float=Decimal to capture 
        # the exact string representation provided by the user.
        cleaned_input = input_str.strip()
        if not cleaned_input.startswith('['):
            cleaned_input = f"[{cleaned_input}]"
        
        values = json.loads(cleaned_input, parse_float=Decimal, parse_int=Decimal)
        
        if not isinstance(values, list):
            raise ValueError("Input must be a JSON-formatted list of numbers.")

        result_bytes = encode_fixed_128(values)

        # Output the raw hex result
        print("0x" + result_bytes.hex())
            
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
