import numpy as np
import math
from qiskit import QuantumRegister, ClassicalRegister, QuantumCircuit, Aer, assemble
from qiskit.visualization import plot_histogram, plot_bloch_vector
from math import sqrt, pi
from qiskit.visualization import plot_state_qsphere
from qiskit import *


I2 = np.matrix([[1,0], [0,1]])
sqr2 = 1/math.sqrt(2)
H2 = np.matrix([[sqr2, sqr2], [sqr2, -sqr2]])
CNOT = np.matrix([[1,0,0,0], [0,1,0,0], [0, 0, 0, 1], [0, 0, 1, 0]])
np.set_printoptions(formatter={'float': '{: 0.1f}'.format})
UNOT = np.matrix([[0, 1], [1, 0]])

U1 = np.kron(np.kron(I2, H2), I2)
U2 = np.kron(I2, CNOT)
print (np.tensordot(I2, CNOT))

U3 = np.kron(CNOT, I2)
U4 = np.kron(np.kron(H2, I2), I2)
U5 = np.kron(I2, CNOT)
U6 = np.kron(np.kron(I2, I2), H2)
U7 = np.kron(np.kron(I2,I2), UNOT)
U8 = np.kron(np.kron(I2, I2), H2)

U =U1*U2

print (U)

qreg_q = QuantumRegister(3, 'q')
creg_c = ClassicalRegister(1, 'c')
circuit = QuantumCircuit(qreg_q, creg_c)

# circuit.initialize([1/2, sqrt(3)/2], 0)
# circuit.x(qreg_q[0])

# circuit.initialize([sqrt(3)/2,  1/2 ], 0)

circuit.barrier(qreg_q[0], qreg_q[1], qreg_q[2])
circuit.h(qreg_q[1])
circuit.cx(qreg_q[1], qreg_q[2])

circuit.draw()

sim = Aer.get_backend('unitary_simulator')

job = execute(circuit, sim)
result = job.result()
a  = result.get_unitary(decimals=1).to_matrix()

print (a)
