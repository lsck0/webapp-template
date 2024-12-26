#!/bin/bash
# This script sets up a new project based on the Webapp Template.

read -rp "Project Name: " project_name
if [ -z "$project_name" ]; then
    echo "[ERROR] Project Name is required."
    exit 1
fi
if [ -d ../"$project_name" ]; then
    echo "[ERROR] Project already exists."
    exit 1
fi

read -rp "Project Owner: " project_owner
if [ -z "$project_owner" ]; then
    echo "[ERROR] Project Owner is required."
    exit 1
fi

read -rp "EC2 Key Pair Name: " ec2_key_pair_name
if [ -z "$ec2_key_pair_name" ]; then
    echo "[ERROR] EC2 Key Pair Name is required."
    exit 1
fi

# ------------------------------------------------------------------------------
echo "[INFO] Cleaning up the template."
git clean -fdx >/dev/null 2>/dev/null

# ------------------------------------------------------------------------------
echo "[INFO] Copying the template."
mkdir ../"$project_name"
cp -r . ../"$project_name"
cd ../"$project_name"

# ------------------------------------------------------------------------------
echo "[INFO] Setting up ${project_name}."
rm use.sh
rm -rf .git
git init >/dev/null 2>/dev/null

dash_case_project_name=$(echo $project_name | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr '_' '-')

declare -a subs=(
    "s/Webapp Template/$project_name/g"
    "s/webapp-template/$dash_case_project_name/g"
    "s/--project-name wat/--project-name ${dash_case_project_name}/g"
    "s/wat-/${dash_case_project_name}-/g"
    "s/Luca Sandrock/$project_owner/g"
    "s/lsandrock/$ec2_key_pair_name/g"
)
for sub in "${subs[@]}"; do
    find . -type f | xargs -i sed -i "$sub" {}
done

cp ../webapp-template/README.md .
sed -i "s/Webapp Template/$project_name/g" README.md
#sed -i "/to use this template for a new project/d" README.md

# ------------------------------------------------------------------------------
echo "[INFO] Committing the changes."
git add .
git commit -m "Initial commit." > /dev/null

# ------------------------------------------------------------------------------
echo "[INFO] ${project_name} created at" $(pwd)

# ------------------------------------------------------------------------------
#echo "[INFO] Entering ${project_name} shell."
#nix develop
