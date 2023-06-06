from saleae import automation
import os
import os.path

# Connect to the running Logic 2 Application on port `10430`
with automation.Manager.connect(port=10430) as manager:

    # Configure the capturing device to record on digital channel 0
    # with a sampling rate of 10 MSa/s, and a logic level of 3.3V.
    device_configuration = automation.LogicDeviceConfiguration(
        enabled_digital_channels=[0],
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
        os.makedirs(output_dir)

        # Export raw digital data to a CSV file
        capture.export_raw_data_csv(directory=output_dir, digital_channels=[0])

        # Finally, save the capture to a file
        capture_filepath = os.path.join(output_dir, 'example_capture.sal')
        capture.save_capture(filepath=capture_filepath)
