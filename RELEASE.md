# Creating a Release

1. Run the [release script](./release.sh): `./release.sh v[X.Y.Z]`
2. Push the tags: `git push origin HEAD && git push origin v[X.Y.Z]`
3. Wait for [Continuous Deployment](https://github.com/n3tw0rth/jired/actions) workflow to finish.
