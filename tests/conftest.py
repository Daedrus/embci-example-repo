import pytest

def pytest_addoption(parser):
    parser.addoption(
        "--capture_name",
        action="store",
        default="digital.csv",
        help="name of Saleae capture file"
    )
    parser.addoption(
        "--saleae_capture_channel",
        action="store",
        default=0,
        help="Saleae logic analyzer capture channel"
    )

@pytest.fixture
def capture_name(request):
    return request.config.getoption("--capture_name")

@pytest.fixture
def saleae_capture_channel(request):
    return int(request.config.getoption("--saleae_capture_channel"))
