import unicodedata2;

def main() -> None:
    ranges = []
    total = 0

    for c in range(0, 0x10FFFF+1):
        ch = chr(c)

        if unicodedata2.name(ch, None) and 'M' in unicodedata2.category(ch):
            if len(ranges) > 0 and ranges[len(ranges) - 1][1] == c - 1:
                ranges[len(ranges) - 1] = (ranges[len(ranges) - 1][0], c)
            else:
                ranges.append((c, c))

            total += 1

    for (a, b) in ranges:
        if a == b:
            print(f"| 0x{a:X}")
        else:
            print(f"| 0x{a:X}..=0x{b:X}")

    print(f"Found {total} combining characters")
    print(f"Unicode version: {unicodedata2.unidata_version}")

if __name__ == '__main__':
    main()