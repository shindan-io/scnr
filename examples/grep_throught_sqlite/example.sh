
scnr scan -i ../../_samples -f *.db | grep -B 1 -A 3 Islands > actual_output.txt

diff expected_output.txt actual_output.txt
