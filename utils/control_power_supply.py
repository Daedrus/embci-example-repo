import argparse
import pyvisa
import time
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('query', help='Query to send to power supply')
    args = parser.parse_args()

    rm = pyvisa.ResourceManager()
    rnd_320_ka3305p = rm.open_resource('ASRL/dev/ttyPowerSupply')

    # Some of the commands we use don't return anything so just handle
    # them like this for now.
    if args.query.startswith('VSET') or args.query.startswith('OUT'):
        rnd_320_ka3305p.write(args.query)
    else:
        print(rnd_320_ka3305p.query(args.query))

    # Sleep for a second to allow the power supply to do its thing
    time.sleep(1)

if __name__ == '__main__':
    main()

# TODO: Implement Python class for handling all power supply commands
