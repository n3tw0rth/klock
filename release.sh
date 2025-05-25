#!/usr/bin/env bash

if ! command -v git-cliff &>/dev/null; then
  echo "git-cliff is not installed. Run 'cargo install git-cliff' to install it, otherwise the changelog won't be generated"
  exit 1
fi

if ! command -v typos &>/dev/null; then
  echo "typos is not installed. Run 'cargo install typos-cli' to install it, otherwise the typos won't be fixed"
  exit 1
fi

if [ -z "$1" ]; then
	echo "Please provide a tag."
	echo "Usage: ./release.sh v[X.Y.Z]"
	exit
fi

echo "Preparing $1..."

# Update the version
msg="# managed by release.sh"
sed -E -i '' "s/^version = \".*\"( # managed by release.sh)?/version = \"${1#v}\" # managed by release.sh/" Cargo.toml

# Update the changelog
git-cliff --config cliff.toml --tag "$1" >CHANGELOG.md
git add -A && git commit -m "chore(release): prepare for $1"

# Generate a changelog for the tag message
export GIT_CLIFF_TEMPLATE="\
	{% for group, commits in commits | group_by(attribute=\"group\") %}
	{{ group | upper_first }}\
	{% for commit in commits %}
		- {% if commit.breaking %}(breaking) {% endif %}{{ commit.message | upper_first }} ({{ commit.id | truncate(length=7, end=\"\") }})\
	{% endfor %}
	{% endfor %}"

changelog=$(git-cliff --config cliff.toml --unreleased --strip all)

git tag -a "$1" -m "Release $1" -m "$changelog"
git tag -v "$1"

echo "Done!"
echo "Now push the commit (git push) and the tag (git push --tags)."
