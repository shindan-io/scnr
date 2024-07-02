
scnr scan -i ../../_samples -f *.db -b | grep Islands > actual_output.txt

diff expected_output.txt actual_output.txt
