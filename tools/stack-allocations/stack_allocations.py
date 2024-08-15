# use: python stack_allocations.py executable_file num_results

import sys
import subprocess

executable = sys.argv[1]
num = int(sys.argv[2])
output = subprocess.check_output([
    'objdump', '-D', '--prefix-addresses', '--demangle=rust', executable
    ]).decode('utf-8').splitlines()

result = []

for line in output:
    line = ' '.join(line.split())

    if line.endswith(',%rsp') and ' sub $' in line:
        a, b = line.split(' sub $')
        function = ' '.join(a.split('+')[0].split(' <')[1:])
        stack_size = int(b.split(',')[0], 16)
        result.append((stack_size, function))

result.sort(key=lambda x: x[0], reverse=True)

for (stack_size, function) in result[:num]:
	print(f'{stack_size:#x} => {function}')



