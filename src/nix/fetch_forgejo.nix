{ domain, owner, repo, ref, args }:

if ref != null || args ? ref
then fetchTarball { url = "https://${domain}/${owner}/${repo}/archive/${if ref != null then ref else args.ref}.tar.gz"; }
else fetchGit (
  {
    url = "https://${domain}/${owner}/${repo}";
    shallow = true;
  } // args
)
