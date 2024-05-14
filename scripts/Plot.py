class Data :

    budget : int
    efficiency : float
    reachables : int

    def __init__(self, budget, efficiency, reachables):
        self.budget = int(budget)
        self.efficiency = efficiency
        self.reachables = int(reachables)

    def __str__(self):
        return f"Data <budget: {self.budget}, efficiency: {self.efficiency}, reachables: {self.reachables}>"
    
    def __repr__(self):
        return self.__str__()
    

COLOURS_AND_NAMES = [
    ("none", ((0,0,0), "Non attaqué")),
    ("random", ((0,0,250), "Statique Aléatoie")),
    ("full_random", ((50,0,150), "Aléatoire complet")),
    ("random_moving", ((0,150,250), "Aléatoire marche")),
    ("betweenness", ((0,200,0), "Centralité statique")),
    ("betweenness_moving", ((0,150,0), "Centralité dynamique")),
    ("longitude", ((200,0,0), "Longitude")),
    ("longitude_moving", ((150,0,0), "Longitude moving")),
    ("latitude", ((250,0,0), "Latitude")),
    ("identifier", ((200,0,200), "Identifier")),
    ("max_lanes", ((200,200,0), "Max lanes")),
]

    
import argparse as arg
import math
import numpy as np 
import matplotlib.pyplot as plt 
from matplotlib.pyplot import cm

import os

import matplotlib.pyplot as plt
import matplotlib.scale as mscale
import matplotlib.transforms as mtransforms
import numpy as np

def load_file(path):
    """
    str(file path) -> list[Data]
    loads the content of the file at path
    into a list of Data objects
    """
    data = np.reshape(np.loadtxt(path), (-1,3))
    return [Data(*d) for d in data]


def load_data(dir_path):
    """
    str(directory path) -> dict[str, list[list[Data]]]
    loads the result files from a directory generated
    by the launch script into a list of [name, list[Data]]
    """

    # Initialize the results dictionary and get the list of strategies
    results = dict()
    dir_path = "data/" + dir_path
    attacks = [a for a in os.listdir(dir_path) if not (a == "graph" or  "longitude" in a or  "max_lanes" in a or "identifier" in a)]
    # Read the number of links from the graph file
    print(attacks)
    with open(dir_path + "/graph") as f:
        num_links = int(f.readline().split()[0])
    clean_names = set(a.split("__")[0] for a in attacks)

    # Collect the results for each attack type (multiple instances of the same attack type are put in a list)
    for name in clean_names:
        results[name] = []

    for attack in attacks:
        split = attack.split("__")
        clean_name = split[0]
        results[clean_name].append(load_file(dir_path + "/" + attack + "/result"))

    # Find an attack other than "none" to get the length of the data
    for attack in results:
        if attack == "none":
            continue
        if len(results[attack]) > 0:
            budgets = results[attack][0]

    # Extend the "none" data to the length of the other data
    if "none" in results:
        none_data = results["none"][0][0]
        results["none"] = [[Data(other_data.budget, none_data.efficiency, none_data.reachables) for other_data in budgets]]
        results["none"][0] = [Data(0, none_data.efficiency, none_data.reachables)] + results["none"][0] + [Data(num_links, none_data.efficiency, none_data.reachables)]

    # Extend the other data by adding the budget 0 with the efficiency and reachables of the "none" data
    for attack in results:
        if attack == "none":
            continue
        for i in range(len(results[attack])):
            results[attack][i] = [Data(0, none_data.efficiency, none_data.reachables)] + results[attack][i] + [Data(num_links, 0, 0)]
            
    return results


def plot_data(ax, data : dict[list[Data]], attribute) :
    """
    list[ [str, numpy.array[-1,3] ] ] -> None 

    plots the data loaded with the 
    dt_from_dir_fn
    """

    for (attack_type, (colour, name)) in COLOURS_AND_NAMES:
        if attack_type in data:
            x = [d.budget for d in data[attack_type]]
            y = [getattr(d, attribute) for d in data[attack_type]]
            z = [a for a in range(len(x))]
            ax.plot(z, y, color=[c/255 for c in colour], label=name, linewidth=2)
            
            
            z = [a for a in range(len(x))]

            # TODO : fix ticks
            xticks = [str(int(a)) for (i,a) in enumerate(x) if i%5 == 0]
            ax.set_xticks([i for i in range(len(x)) if i%5 == 0])  # set x-ticks on the Axes object
            ax.set_xticklabels(xticks)  # set x-tick labels on the Axes object
        else :
            print(f"Attack type {attack_type} not found in data")

    ax.legend()


def mk_plot(ax, data, attribute, title="Ville", xtitle="", ytitle="") :
    """
    string(directory path) -> None 
    plots from the dir passed
    """
    ax.set_title(title)
    ax.set_xlabel(xtitle.capitalize())
    ax.set_ylabel(ytitle.capitalize())
    
    # Make a copy of the data and average them if they have multiple instances
    data_averaged = dict()
    for attack in data:
        zipped = list(zip(*data[attack]))
        data_averaged[attack] = [Data(
            d[0].budget,
            sum([d[i].efficiency for i in range(len(d))]) / len(d),
            int(sum([d[i].reachables for i in range(len(d))]) / len(d))
        ) for d in zipped]

    plot_data(ax, data_averaged, attribute)

def rand_from_dir(dir_path):
    """
    str(directory path) -> dict[str, list[Data]]
    Loads only the random attacks from the directory
    """

    # Initialize the results dictionary and get the list of strategies
    results = dict()
    dir_path = "data/" + dir_path
    attacks = [a for a in os.listdir(dir_path) if not a == "graph"]
    clean_names = set(a.split("__")[0] for a in attacks)

    # Collect the results for each attack type (multiple instances of the same attack type are averaged)
    for name in clean_names:    
        results[name] = []    
    for attack in attacks:
        split = attack.split("__")
        clean_name = split[0]
        if "random" in clean_name or "none" in clean_name:
            results[clean_name].append(load_file(dir_path + "/" + attack + "/result"))    

    return results

def plot_rand(ax, data, attribute) :
    """
    list[ [str, numpy.array[-1,3] ] ] -> None 

    plots the data loaded with the 
    dt_from_dir_fn
    """

    # Sort the data by attack type
    for (attack_type, (colour, name)) in COLOURS_AND_NAMES:
        if "random" not in attack_type and "none" not in attack_type:
            continue
        if attack_type in data:
            for (i, current_data) in enumerate(data[attack_type]):
                x = [d.budget for d in current_data]
                y = [getattr(d, attribute) for d in current_data]
                z = [a for a in range(len(x))]
                
                if i == 0:
                    ax.plot(z, y, color=[c/255 for c in colour], label=name, linewidth=2)
                else:
                    ax.plot(z, y, color=[c/255 for c in colour], linewidth=2)
                
                z = [a for a in range(len(x))]
                xticks = [str(int(a)) for (i,a) in enumerate(x) if i%5 == 0]
                ax.set_xticks([i for i in range(len(x)) if i%5 == 0])  # set x-ticks on the Axes object
                ax.set_xticklabels(xticks)  # set x-tick labels on the Axes object
        else :
            print(f"Attack type {attack_type} not found in data")

    ax.legend()

def mk_plot_rand(ax, data, attribute, xtitle="", ytitle="",title="") :
    """
    string(directory path) -> None 
    plots from the dir passed
    """
    ax.set_title(title)
    ax.set_xlabel(xtitle.capitalize())
    ax.set_ylabel(ytitle.capitalize())

    data_rand = {k: v for k, v in data.items() if "random" in k or "none" in k}
    plot_rand(ax, data_rand, attribute)

def main():

    parser = arg.ArgumentParser()
    parser.add_argument("path_ville")
    args = parser.parse_args()

    data = load_data(args.path_ville)


    plt.rc('font', size=20)          # controls default text sizes
   
    plt.rc('axes', labelsize=18)    # fontsize of the x and y labels
    plt.rc('xtick', labelsize=9)    # fontsize of the tick labels
    plt.rc('ytick', labelsize=9)    # fontsize of the tick labels
    plt.rc('legend', fontsize=18)    # legend fontsize
    plt.rc('figure', titlesize=18)  # fontsize of the figure title
 
    fig, ((ax1,ax2),(ax3,ax4)) = plt.subplots(2,2)

    #fig.suptitle("Attaques sur " + + " à impact fort", fontsize=25)
    

    mk_plot(ax1, data, "efficiency",ytitle="efficacité", xtitle="Nombre de liens attaqués", title="Efficacité en fonction du nombre de liens attaqués")
    mk_plot(ax3, data, "reachables",ytitle="accessibilité", xtitle="Nombre de liens attaqués", title="Accessibilité en fonction du nombre de liens attaqués")

    mk_plot_rand(ax2, data, "efficiency", ytitle="efficacité", xtitle="Nombre de liens attaqués",title="Efficacité en fonction du nombre de liens attaqués")
    mk_plot_rand(ax4, data, "reachables", ytitle="accessibilité", xtitle="Nombre de liens attaqués", title="Accessibilité en fonction du nombre de liens attaqués")

    fig.set_figheight(15)
    fig.set_figwidth(30)

    fig.show()
    # If plots subdirectory does not exist, create it
    if not os.path.exists("plots"):
        os.makedirs("plots")
    plt.margins(0.1, 0.1)
    fig.savefig("plots/plot_"+args.path_ville+".pdf", format="pdf", dpi=1200, bbox_inches='tight')
    
    
if __name__ == "__main__" : 
    main()

# Exemple d'utilisation :
# python3 scripts/Plot.py Maza