#!/usr/bin/fish
target/release/maze $argv &&\
 target/release/maze $argv &&\
 target/release/maze $argv &&\
 target/release/maze $argv &&\
 for i in maze*svg
   convert $i $i.pdf
 end &&\
 pdfjam --paper a4paper maze-*pdf &&
 pdfxup -ps a4 -o mazes.pdf *pdfjam.pdf &&\
 rm -f maze-*.svg maze-*.pdf