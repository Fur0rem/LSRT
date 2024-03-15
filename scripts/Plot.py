import matplotlib.pyplot as plt
import numpy as np
import argparse as arg
import sys

"""
def plot(x, y, title, xlabel, ylabel, legend):
    plt.plot(x, y, label=legend)
    plt.title(title)
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.legend()
    plt.show()


def plot_dir(dir):

    # Read the data from the file

    dict_x = {}
    dict_y = {}
    with open(dir, 'r') as dir : 

        for file in dir : 
            with open(file, 'r') as f : 
                if

def main() :
    parser = arg.ArgumentParser()
    parser.add_argument("dir") 

    plot(x, y, "", "", "", "") 

if name == "main":
    main()
"""

x = np.loadtxt("melun_0503/Melun_betweenness_centralities_-1_8_2_1000", usecols=range(2))
y = np.loadtxt("melun_0503/Melun_betweenness_centralities_-1_8_2_nbattacked")

x1 = np.array([i[0] for i in x])

#print(x1)
points = []
for cpt in range (len(x1)) : 
    points.append(([y[cpt], x1[cpt]]))

print(points)
plt.xscale("log")
plt.plot( *zip(*points),label="Betweenness centrality")

plt.show()