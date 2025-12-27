{ owner, repo, branch, args }:

if branch != null || args ? branch
then fetchTarball { url = "https://github.com/${owner}/${repo}/archive/refs/heads/${if branch != null then branch else args.branch}.tar.gz"; }
else if args ? tag
then fetchTarball { url = "https://github.com/${owner}/${repo}/archive/refs/tags/${args.tag}.tar.gz"; }
else if args ? commit
then fetchTarball { url = "https://github.com/${owner}/${repo}/archive/${args.commit}.tar.gz"; }
else fetchGit (
  {
    url = "https://github.com/${owner}/${repo}";
    shallow = true;
  } // args
)
