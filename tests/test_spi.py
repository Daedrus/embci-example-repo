from saleae import automation
import os
import os.path
import pytest
import pyvisa


@pytest.fixture(autouse=True)
def get_spi_saleae_capture(capture_name):
    rm = pyvisa.ResourceManager()
    rnd_320_ka3305p = rm.open_resource('ASRL/dev/ttyPowerSupply')

    # Connect to the running Logic 2 Application on port `10430`
    with automation.Manager.connect(port=10430) as manager:

        # Configure the capturing device to record on digital channel 0
        # with a sampling rate of 10 MSa/s, and a logic level of 3.3V.
        device_configuration = automation.LogicDeviceConfiguration(
                enabled_digital_channels=[5, 6, 4, 7],
                digital_sample_rate=10_000_000,
                digital_threshold_volts=3.3,
                )

        # Record 10 seconds of data before stopping the capture
        capture_configuration = automation.CaptureConfiguration(
                capture_mode=automation.TimedCaptureMode(duration_seconds=10.0)
                )

        # Start a capture - the capture will be automatically closed when leaving the `with` block
        with manager.start_capture(
                device_configuration=device_configuration,
                capture_configuration=capture_configuration) as capture:

            rnd_320_ka3305p.write('OUTCH1:1')

            # Wait until the capture has finished
            capture.wait()

            rnd_320_ka3305p.write('OUTCH1:0')

            spi_analyzer = capture.add_analyzer(
                'SPI',
                label=f'SPI Analyzer',
                settings={
                    'MOSI': 5,
                    'MISO': 6,
                    'Clock': 4,
                    'Enable': 7,
                    'Significant Bit': 'Most Significant Bit First (Standard)',
                    'Bits per Transfer': '8 Bits per Transfer (Standard)',
                    'Clock State': 'Clock is High when inactive (CPOL = 1)',
                    'Clock Phase': 'Data is Valid on Clock Trailing Edge (CPHA = 1)',
                    'Enable Line': 'Enable line is Active Low (Standard)',
                })

            # Store output
            output_dir = os.path.join(os.getcwd(), f'output')
            os.makedirs(output_dir, exist_ok=True)

            legacy_export_filepath = os.path.join(output_dir, capture_name)
            capture.legacy_export_analyzer(
                filepath=legacy_export_filepath,
                analyzer=spi_analyzer,
                radix=automation.RadixType.HEXADECIMAL
            )


def test_spi(capture_name):
    with open('output/' + capture_name) as f:
        # The first line is "Time [s],Packet ID,MOSI,MISO"
        spi_bytes = f.read().splitlines()[1:]

        # For now, just check that the BMP280 chip identification number is read out correctly
        # (rpi puts 0xD0 on the MOSI line and gets back 0x58 on the MISO line)
        assert spi_bytes[0].split(',')[2] == '0xD0'
        assert spi_bytes[1].split(',')[3] == '0x58'
