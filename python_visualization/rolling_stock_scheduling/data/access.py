import importlib.resources as resources
from pathlib import Path


class PkgDataAccess:
    def __init__(self) -> None:
        pass

    @staticmethod
    def locate_response() -> Path:
        data_folder = resources.files("rolling_stock_scheduling.data")
        file_path = data_folder / "output_small_test_input.json"
        return file_path
