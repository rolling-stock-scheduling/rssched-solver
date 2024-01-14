from rolling_stock_scheduling.data.access import PkgDataAccess
from rolling_stock_scheduling.io.reader import import_response


def test_import_response():
    response = import_response(PkgDataAccess.locate_response())
    assert len(response.schedule) == 4
