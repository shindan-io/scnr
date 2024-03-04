
scnr scan -i ../../_samples -f *.db | grep Islands > expected_output.txt

diff expected_output.txt actual_output.txt
