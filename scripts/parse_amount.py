#!python3

import sys
from decimal import Decimal, getcontext

def parse_fixed_uint128(input_str, decimals=18):
    """
    Parses a single uint128 value. Handles:
    1. Standard Decimal strings (prioritized).
    2. ABI-encoded 32-byte hex words (Big-Endian).
    3. Raw 16-byte hex strings (Little-Endian).
    """
    # Set precision for the intricate decimal math
    getcontext().prec = 50 
    
    input_str = input_str.strip().lower()
    has_hex_prefix = input_str.startswith('0x')
    if has_hex_prefix:
        input_str = input_str[2:]
    
    try:
        # Check if we should treat this as hex
        # We treat as hex if it has a prefix OR contains letters a-f
        is_hex = has_hex_prefix or any(c in 'abcdef' for c in input_str)
        
        if is_hex:
            # Case A: 32-byte ABI word (64 hex chars)
            # This is standard Big-Endian padding for a single return value.
            if len(input_str) == 64:
                val = int(input_str, 16)
            
            # Case B: 16-byte raw u128 (32 hex chars)
            # This is likely the Little-Endian format extracted from a vector.
            elif len(input_str) == 32:
                raw_bytes = bytes.fromhex(input_str)
                val = int.from_bytes(raw_bytes, byteorder='little')
            
            # Case C: Other hex
            else:
                val = int(input_str, 16)
        else:
            # Case D: It's just a standard decimal string
            # We use a deep breath and standard int conversion
            val = int(input_str)

        # Convert to fixed point
        fixed_val = Decimal(val) / Decimal(10**decimals)
        return f"{fixed_val:.18f}"

    except ValueError:
        return None

def main():
    input_data = ""

    # Check argv first, then check if we are piped data via stdin
    if len(sys.argv) > 1:
        input_data = sys.argv[1]
    elif not sys.stdin.isatty():
        input_data = sys.stdin.read().strip()

    if not input_data:
        print("Usage: python3 parse_uint128.py <hex_or_decimal>")
        sys.exit(0)

    # In case of multiple lines, we take the last line (the result)
    # to bypass headers or noise.
    if "\n" in input_data:
        input_data = input_data.splitlines()[-1]

    result = parse_fixed_uint128(input_data)

    if result:
        print(f"Parsed Value: {result}")
    else:
        print("Error: Could not parse input as a uint128.")
        sys.exit(1)

if __name__ == "__main__":
    main()
