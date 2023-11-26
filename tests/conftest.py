import pytest

def pytest_addoption(parser):
    parser.addoption(
        "--capture_file",
        action="store",
        default="digital.csv",
        help="name of Saleae capture file"
    )

@pytest.fixture
def capture_file(request):
    return request.config.getoption("--capture_file")
