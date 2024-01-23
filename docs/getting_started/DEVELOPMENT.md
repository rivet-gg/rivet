# Developing

> 🚨 **READ BEFORE PROCEEDING** 🚨
>
> We encourage developers to use the self-hosted version of Rivet, but we're
> still working on improving the self-hosted developer experience.
>
> At the moment, all the services need to be built from scratch and deployed
> to a fully loaded Kubernetes cluster, which requires a beefy Linux machine.
> (Ideally 8 CPU cores, 32 GB RAM to build the required components for the
> cluster & run Rust Analyzer at the same time.)
>
> Once running, Rivet itself is very lightweight since it's all Rust. If you
> want to help make Rivet easier to run, please reach out and we can help
> provide guidance on [Discord](https://discord.gg/BG2vqsJczH) when
> implementing the following issues:
>
> -   https://github.com/rivet-gg/rivet/issues/153
> -   https://github.com/rivet-gg/rivet/issues/155
> -   https://github.com/rivet-gg/rivet/issues/157
> -   https://github.com/rivet-gg/rivet/issues/154
> -   https://github.com/rivet-gg/rivet/issues/156
> -   https://github.com/rivet-gg/rivet/issues/157

## Prerequisites
-   Debian 11 (other Linux distros untested)
    -   Accessible from public IP
    -   Recommended to configure your [Firewalls](/docs/getting_started/DEVELOPMENT_FIREWALLS.md)
-   A [Cloudflare account](https://developers.cloudflare.com/fundamentals/get-started/setup/add-site/) (free)
    - With a top-level domain.
-   A [Linode](https://login.linode.com/signup) or [OneProvider](https://oneprovider.com/onecloud/pricing) account
    - On Linode you would require a _Linode 32 GB Shared CPU VM_ for __$192/month__
    - On OneProvider you would require a _High Performance Instance with 8 vCPU & 32 GB RAM_ for __€45/month__
-   A [SendGrid account](https://sendgrid.com/)

## Step 1: Install dependencies
- [Docker](https://docs.docker.com/engine/install/) _is required to run Rivet services._
- [Nix package manager](https://nixos.org/download.html) _is required to set up the development environment._
- [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) _is required to clone Rivet's source code._

## Step 2: Clone the repository
> **Tip**:
> Warp may have issues with the Nix installer since it does not use the default shell. [Read more.](https://docs.warp.dev/features/ssh)
```
git clone https://github.com/rivet-gg/rivet.git
```

## Step 3: Boot the cluster
> **Tip**:
> See the `namespaces/dev.toml` and `secrets/dev.toml` file to see the generated namespace configs.
```
nix-shell --run "bolt init dev --yes"
```
This will:
1. Prompt you for parameters to generate your cluster's config
2. Provision required infrastructure for the cluster

## Step 4: Boot the Rivet Hub
Now, you have to start the hub frontend.

1. Clone the [Rivet Hub](https://github.com/rivet-gg/hub) with
2. Create a file called `.env` in the root of the `hub` repo
    - Add `RIVET_API_ENDPOINT=http://[YOUR DOMAIN]:8080` to the top of the file (this is commonly http://localhost:8080)
    - For more guidance follow the docs [here](https://github.com/rivet-gg/hub/blob/main/README.md)
3. Start the hub; `yarn start`
4. Visit http://localhost:5080
5. Register an account with your email

## Step 5: Promote yourself to admin

```
nix-shell --run "bolt db sh db-user --query 'UPDATE users SET is_admin = true'"
```

You may need to clear the local cache for this change to appear. ([Related issue](https://github.com/rivet-gg/rivet/issues/152))

_This command sets all users to admin. We're assuming you're the only user in the cluster. Do not run this command again._

## Step 6: Create a developer group

You should now see a "Create Group" button on the hub. Proceed to create a group and start developing!

## Next steps

Read more about [working with Bolt](/docs/libraries/bolt/README.md) or see other [helpful docs](/README.md#-documentation-overview)
