{ sourcesJSON }:

with builtins;
let
  sources = fromJSON sourcesJSON;

  filterAttrsBy = allowed: set: removeAttrs set (filter (name: !elem name allowed) (attrNames set));

  fetchGitArgs = [
    "url"
    "name"
    "rev"
    "ref"
    "submodules"
    "exportIgnore"
    "shallow"
    "lfs"
    "allRefs"
    "verifyCommit"
    "publicKey"
    "keytype"
    "publicKeys"
  ];

  fetchTarballArgs = [
    "url"
    "sha256"
  ];

  fetchGitSource = source: fetchGit ({
    submodules = true;
    shallow = true;
  } // (filterAttrsBy fetchGitArgs source));

  fetchTarballSource = source: fetchTarball (filterAttrsBy fetchTarballArgs source);

  fetchForgejoSource = source:
    if source ? ref
    then fetchTarball { url = "https://${source.domain}/${source.owner}/${source.repo}/archive/${source.ref}.tar.gz"; }
    else fetchGitSource (source // { url = "https://${source.domain}/${source.owner}/${source.repo}"; });

  fetchGithubSource = source:
    if source ? branch
    then fetchTarballSource (source // { url = "https://github.com/${source.owner}/${source.repo}/archive/refs/heads/${source.branch}.tar.gz"; })
    else if source ? tag
    then fetchTarballSource (source // { url = "https://github.com/${source.owner}/${source.repo}/archive/refs/tags/${source.tag}.tar.gz"; })
    else if source ? commit
    then fetchTarballSource (source // { url = "https://github.com/${source.owner}/${source.repo}/archive/${source.commit}.tar.gz"; })
    else fetchGitSource (source // { url = "https://github.com/${source.owner}/${source.repo}"; });

  fetchSource = source: (
    if source.fetchType == "git" then
      fetchGitSource source
    else if source.fetchType == "tarball" then
      fetchTarballSource source
    else if source.fetchType == "forgejo" then
      fetchForgejoSource source
    else if source.fetchType == "github" then
      fetchGithubSource source
    else
      throw ("Unknown fetcher type: " + source.fetchType)
  );

in replaceStrings [ " " ] [ "\n" ] (toString (map fetchSource sources))
