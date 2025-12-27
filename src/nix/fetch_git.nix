{ url, args }:

fetchGit (
  {
    inherit url;
    submodules = true;
    shallow = true;
  } // args
)
