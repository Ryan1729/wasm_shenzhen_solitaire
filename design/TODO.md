* Initial setup/pruning of `evolving-games` version of the "engine".
* Get the sprite data from the pico-8 cartridge into the rust code somehow.
    * In this case, just writing numbers into a `Vec` seems like the best way
* Implement pico-8's `spr` and `sspr` functions.
    * or at least the functionality the game uses
* Setup `_init`, `_update` and `_draw` equivalents.
* Port over lua code in the most direct way possible
* optional: Refactor code to be more "rustic".