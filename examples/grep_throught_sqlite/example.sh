
scnr scan -i ../../_samples -f *.db | grep -B 2 -A 2 Islands > actual_output.txt

diff expected_output.txt actual_output.txt
