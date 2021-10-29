#!/bin/sh

#
# You can use this script to format and commit the code all at once
#
#

cargo fmt

if [ $? -eq 0 ]
then
    echo "ran cargo fmt!!!!"
else
    exit 1
fi


git update-index --again

git commit