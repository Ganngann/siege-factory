# TODO
- **Fog of war** — les tiles non explorées sont masquées ; le joueur découvre la carte en progressant.
- **Belt virage 45°** — ajouter des courroies d'angle (courbes) en plus des courroies droites.
- **Générateur électrique** — le générateur doit stocker du combustible et le consommer pour produire de l'électricité (énergie).
- **Menu de construction progressif** — les bâtiments ne doivent apparaître dans le menu de construction que lorsque le joueur les a débloqués (recherche, progression).
- ~~les ressources doivent etre consomées quand le batiement est construit.~~
- ~~pour ouvrire la fenetre d'ui d'un batiment il faut s'approcher du batiment et interagire.~~
- ~~on dois pouvoir prendre des ressources dans l'inventaire d'un batiment et les mettre dans notre inventaire et vice versa~~
- on dois pouvoir ramasser des ressources qui sont sur des belt
- pour miner des ressources, on dois rester appuié, et ca en mine une tout les x temps
- on ne dois pouvoir voire un gisement de ressource que si on a découvert une techno ou un truc comme ca



# Fait
- ~~**[BUG] Mine → Dépôt** — clic bâtiment prioritaire sur le dépôt.~~ *FIX: ordre inversé dans `building_inspect_click` (inspect.rs)*
- ~~**Buildings pleins** — belt laisse l'item sur place si inventaire plein (belt.rs:277), prod skip si capacity atteinte (production.rs:48).~~
- ~~**[CAMERA]** la caméra suit le perso automatiquement.~~ *`camera_follow_player` (player.rs)*
- ~~**Perso mine (touche E)** les dépôts adjacents, commence avec 0 ressources.~~ *`player_mine` + `start_ore = 0`*
- ~~**Inventaire ouvrable (touche I)** remplace le HUD haut.~~ *`toggle_inventory_panel` (player.rs)*
