import argparse as arg
import numpy as np 
import matplotlib.pyplot as plt 
from matplotlib.pyplot import cm

import os

def load_file(path):
    """
    str(file path) -> numpy.array[-1,3]

    loads the content of the file at path 
    into a 3*x numpy aray 
    """
    return np.reshape(np.loadtxt(path), (-1,3))

def dt_from_dir(dir_path): 
    """
    str(directory path) -> list[ [str, numpy.array[-1,3] ] ]
    loads the result files from a directory generated 
    by the launch script into a list of [name, np.array]
    """
    res_int = dict()
    dir_path = "data/" + dir_path
    for name in os.listdir(dir_path): 
        if not name == "graph" : 

            split = name.split("_")
            clean_name = name 
            if len(split) > 2 : 
                clean_name = split[0] + " " + split[1]
            else : 
                clean_name = split[0]
            
            if clean_name not in res_int : 
                res_int[clean_name] = [load_file(dir_path + "/" + name + "/result")]
            else : 
                res_int[clean_name].append(load_file(dir_path + "/" + name + "/result"))
    
    res_f = dict()
    for i in res_int.keys() : 
        res_f[i] = sum(res_int[i]) / len(res_int[i])

    return res_f


def plot_data(ax, data,y_index=1): 
    """
    list[ [str, numpy.array[-1,3] ] ] -> None 

    plots the data loaded with the 
    dt_from_dir_fn
    """
    for i in data.keys() :

        x = [ j[0]for j in data[i]]
        y = []
        if y_index == 1 : 
            y = [ j[y_index] * 1000 for  j in data[i] ]
        else :
            y = [ j[y_index] for  j in data[i] ]

        ax.plot(x,y,label=i)

    ax.legend()
    ax.set_xscale("log")
    #ax.show()
    
    #ax.savefig("res")


def mk_plot(ax, dir_name, y_index=1) : 
    """
    string(directory path) -> None 
    plots from the dir passed
    """
    print(dir_name.split("/"[-1]))
    ax.set_title( dir_name.split("/")[-1])

    if y_index==1  : 
        ax.set_ylabel("efficiency in permille")
    else : 
        ax.set_ylabel("reachabillity")

    ax.set_xlabel("number of attacked links")

    plot_data(ax,dt_from_dir(dir_name),y_index)


def rand_from_dir(dir_path) : 
    """
    str(directory path) -> 
    loads the random result files from a directory generated 
    by the launch script into a dict()
    """
    res_int = dict()
    dir_path = "data/" + dir_path
    for name in os.listdir(dir_path): 
        if not name == "graph" : 

            split = name.split("_")
            clean_name = name 
            if len(split) > 2 : 
                clean_name = split[0] + " " + split[1]
            else : 
                clean_name = split[0]

            if "random" in clean_name : 
                if clean_name not in res_int : 
                    res_int[clean_name] = [load_file(dir_path + "/" + name + "/result")]
                else : 
                    res_int[clean_name].append(load_file(dir_path + "/" + name + "/result"))
    
  

    return res_int

def plot_rand(ax, data, y_index=1): 

    col_wheel = iter(cm.rainbow(np.linspace(0, 1, 6)))#pour changer de couleur


    # print(data)
    for i in data.keys() :
        c = next(col_wheel)
        for values in data[i]:

            # print(values)

            x = [ j[0]for j in values]
            y = []
            if y_index == 1 : 
                y = [ j[y_index] * 1000 for  j in values ]
            else :
                y = [ j[y_index] for  j in values ]
            ax.plot(x,y, color=c)

        ax.plot([],[],label=i, color=c)

    ax.legend()
    ax.set_xscale("log")

def mk_plot_rand(ax, city_name, y_index=1) : 
    """
    string(directory path) -> None 
    plots from the dir passed
    """
    ax.set_title(city_name)

    if y_index==1  : 
        ax.set_ylabel("efficiency in permille")
    else : 
        ax.set_ylabel("reachabillity")

    ax.set_xlabel("number of attacked links")

    plot_rand(ax,rand_from_dir(city_name),y_index)

def main():

    parser = arg.ArgumentParser()
    parser.add_argument("path_ville")
    args = parser.parse_args()

    fig, (ax1,ax2) = plt.subplots(1,2)
    fig.suptitle("Attaques sur " + args.path_ville)

    mk_plot(ax1, args.path_ville)

    mk_plot_rand(ax2, args.path_ville)

    #fig.plot()

    fig.set_figheight(15)
    fig.set_figwidth(30)

    fig.show()
    # If plots subdirectory does not exist, create it
    if not os.path.exists("plots"):
        os.makedirs("plots")
    fig.savefig("plots/plot_"+args.path_ville)

    
if __name__ == "__main__" : 
    main()

# Exemple d'utilisation :
# python3 Plot.py Maza