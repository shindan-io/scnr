cd ../../py_scnr
source .venv/bin/activate

cd ../examples/iter_from_python
python3 -m example > actual_output.txt

diff expected_output.txt actual_output.txt
