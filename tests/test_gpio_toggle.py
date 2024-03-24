from saleae import automation
import os
import os.path
import pytest
import itertools
import shutil
from more_itertools import sliding_window


@pytest.fixture(autouse=True)
def get_raw_saleae_capture(capture_file, saleae_capture_channel):

    # Connect to the running Logic 2 Application on port `10430`
    with automation.Manager.connect(port=10430) as manager:

        # Configure the capturing device to record on digital channel 0
        # with a sampling rate of 10 MSa/s, and a logic level of 3.3V.
        device_configuration = automation.LogicDeviceConfiguration(
                enabled_digital_channels=[saleae_capture_channel],
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

            # Wait until the capture has finished
            capture.wait()

            # Store output
            output_dir = os.path.join(os.getcwd(), f'output')
            os.makedirs(output_dir, exist_ok=True)

            # Export raw digital data to a CSV file. The CSV file is called
            # digital.csv by default and there's no way to change the name
            capture.export_raw_data_csv(directory=output_dir, digital_channels=[saleae_capture_channel])

            # So we rename it ourselves
            shutil.move('output/digital.csv', 'output/' + capture_file)


def test_gpio_toggle(capture_file):
    with open('output/' + capture_file) as f:
        # The first line is "Time [s],Channel 0"
        # The second and last lines might have the same state as their subsequent/preceeding lines
        # lines are of the format 'timestamp,state', e.g.: '0.648999600,1'
        pin_state_lines = f.read().splitlines()[2:-1]

        print("Found gpio pin in following states:", pin_state_lines)

        # If there are no samples something went wrong
        assert len(pin_state_lines) != 0

        # [('1', '0'), ('0', '1'), ('1', '0') ... ]
        pin_state_transitions = sliding_window(
                map((lambda s: s.split(',')[1]), pin_state_lines),
                2)

        # Check that all transitions are from 0 to 1 or from 1 to 0
        assert all(map((lambda transition: transition in [('1', '0'), ('0', '1')]), pin_state_transitions))

