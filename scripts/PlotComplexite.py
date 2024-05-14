import numpy as np
import matplotlib.pyplot as plt
import matplotlib.cm as cm

Saints = (183,83,[0.608,0.709,0.769,0.854,0.420])
Verzuolo = (555, 251, [10.746,12.116,14.135,20.521,5.134])
#Coulommiers = (1236, 580, [117.046, 152.621, 210.718, 315.152, 37.480])
Melun = (1370, 662, [154.285, 250.117, 305.731, 517.251, 48.407])
Creteil = (2317, 1089, [510.342, 728.329, 1222.044, 2129.226, 185.871])
Villes = [Saints, Verzuolo, Melun, Creteil]

_1_pourcent = [v[2][0] for v in Villes]
_25_pourcent = [v[2][1] for v in Villes]
_50_pourcent = [v[2][2] for v in Villes]
_75_pourcent = [v[2][3] for v in Villes]
_100_pourcent = [v[2][4] for v in Villes]


# Saints = [1, 2, 10]
# Guerard = [5, 6, 20]
# Coulommiers = [3, 4, 15]
Villes_names = ['Saints', 'Verzuolo', 'Melun', 'Creteil']

# Define the label locations
ind = np.arange(len(Villes))

# Define the width of the bars
width = 0.15

fig, ax = plt.subplots(figsize=(13, 8))  # Set width to 10 inches and height to 6 inches


# Create the bars
percentages = [_1_pourcent, _25_pourcent, _50_pourcent, _75_pourcent, _100_pourcent]
labels = ['1%', '25%', '50%', '75%', '99%']
ax.set_yscale('linear')

COLOR_YELLOW = (1, 0.8, 0.1)
COLOR_RED = (0.9, 0.1, 0)

for i, (percentage, label) in enumerate(zip(percentages, labels)):
    #mix between yellow and red
    color = tuple(np.array(COLOR_YELLOW) + (np.array(COLOR_RED) - np.array(COLOR_YELLOW)) * i/4)
    rects = ax.bar(ind + i*width - (width * len(Villes)/2), percentage, width, label=label, color=color)

def fn(x) :
    x_powf = pow(x, 2.7)
    return x_powf/10000000 * 5

for v in Villes :
    for i in range(len(v[2])) :
        print(f"Complexité de {v[0]} nodes et {v[1]} edges pour {labels[i]} : {fn(v[0])} comparé à {v[2][i]}")

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Temps (s)', size=20)
ax.set_yticks(np.arange(0, 2500, 250))
ax.tick_params(axis='y', labelsize=15)  # Change fontsize to 15
ax.set_title('Temps en fonction de la taille de la ville et du pourcentage de liens attaqués', size=25)
index = [f"{Villes_names[i]} \n {v[0]} noeuds, {v[1]} liens" for i, v in enumerate(Villes)]
ax.set_xticks(ind)
ax.set_xticklabels(index, size=17)  # Set x-axis labels as indices

# Add the legend with title "Pourcentage de liens attaqués"
ax.legend(title='Pourcentage de liens attaqués', loc='upper left', fontsize=15, title_fontsize=15)
fig.tight_layout()
fig.savefig('Complexite.pdf', bbox_inches='tight')