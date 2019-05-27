#!/usr/bin/fish

for i in *.yaml
  set name (basename "$i" ".yaml")
  echo "$name ..."
  yj -o $name.json $i
end
