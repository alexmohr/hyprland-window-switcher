#!/bin/bash

# Define the list of items and their corresponding hidden values
items=("Option 1|hidden_value_1" "Option 2|hidden_value_2" "Option 3|hidden_value_3")

# Show the menu with only the visible parts of the string
selected_item=$(printf "%s\n" "${items[@]}" | cut -d'|' -f1 | wofi --dmenu)

# Now get the hidden value corresponding to the selected visible item
hidden_value=$(echo "${items[@]}" | grep "$selected_item" | cut -d'|' -f2)

# Output the hidden value
echo "Selected hidden value: $hidden_value"
