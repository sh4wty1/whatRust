# Fork sh4wty1/whatRust

Fork pessoal do [karem505/whatRust](https://github.com/karem505/whatRust), mantido só para **Windows e Linux**.

## O que muda em relação ao original

- **Updater aponta para este fork.** O app só baixa atualizações dos releases de `sh4wty1/whatRust`
  (`REPO` e `ASSET_PREFIX` em `src-tauri/src/updater.rs`). Nada chega do original sem passar por aqui.
- **Instaladores** (`install.ps1`, `install.sh`) baixam deste fork. O `install.sh` só instala no Linux.
- **CI e release** compilam só Windows (`.exe`/`.msi`) e Linux (`.AppImage`/`.deb`). macOS e Flatpak saíram
  dos workflows.
- **`upstream-watch`**: workflow semanal que abre uma issue com o que mudou no original (veja abaixo).

O código de macOS e Flatpak continua no repositório, inativo (não compila no Windows/Linux por causa dos
`#[cfg(target_os = "macos")]`). Ele foi mantido de propósito: apagar esses arquivos geraria conflito em
quase todo merge vindo do original.

O identificador do app continua `com.karem.whatrust`: as configurações e o login ficam na mesma pasta
que a versão original usaria. Não instale as duas versões no mesmo PC.

## Instalar

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/sh4wty1/whatRust/master/install.ps1 | iex
```

**Linux** (AppImage em `~/.local/bin`, com atalho no menu):

```sh
curl -fsSL https://raw.githubusercontent.com/sh4wty1/whatRust/master/install.sh | sh
```

Ou baixe o `.exe`, `.msi`, `.AppImage` ou `.deb` direto da
[página de releases](https://github.com/sh4wty1/whatRust/releases/latest).

## Acompanhar o original

Toda segunda-feira o workflow `upstream-watch` compara este fork com o original e mantém **uma** issue
aberta com a label `upstream`, listando os commits que ainda não estão aqui, com link para cada diff.
Para rodar na hora: aba **Actions → upstream-watch → Run workflow**, ou:

```sh
gh workflow run upstream-watch -R sh4wty1/whatRust
```

Commits trazidos com `cherry-pick` saem da lista automaticamente. Quando o fork alcança o original,
a issue é fechada.

> O GitHub desliga workflows agendados depois de 60 dias sem commits no repositório (ele avisa por
> e-mail). Para religar: **Actions → upstream-watch → Enable workflow**.

### Trazer mudanças do original

```sh
git remote add upstream https://github.com/karem505/whatRust.git   # só na primeira vez
git fetch upstream

git log --oneline master..upstream/master     # o que tem de novo
git diff master...upstream/master             # revisar o código antes

git merge upstream/master                     # trazer tudo
# ou
git cherry-pick <commit>                      # trazer só um commit
```

Num merge, os conflitos esperados são nos arquivos que este fork alterou: `src-tauri/src/updater.rs`
(mantenha `sh4wty1/whatRust`), `install.sh`, `install.ps1`, `README.md` e os workflows em
`.github/workflows/` (mantenha sem macOS/Flatpak). Se o original alterar `.github/workflows/flatpak.yml`,
o git avisa que o arquivo foi apagado aqui: mantenha apagado (`git rm .github/workflows/flatpak.yml`).

## Publicar um release

O app instalado só se atualiza quando existe um release novo neste fork.

1. Atualize a versão em `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` (e rode `cargo update -p whatrust`
   dentro de `src-tauri` para o `Cargo.lock` acompanhar). Ao trazer um release do original, o merge já faz isso.
2. Opcional: escreva as notas em `docs/release-notes/vX.Y.Z.md` (vira o texto do release).
3. Faça commit e push para `master`.
4. Dispare o release:

   ```sh
   gh workflow run release -R sh4wty1/whatRust -f tag=vX.Y.Z
   ```

O workflow compila Windows e Linux, gera o `SHA256SUMS` (o updater recusa qualquer download que não
bata com ele) e publica o release.

Versões: use o mesmo número do original quando só trouxer o release dele. Se publicar uma correção só
sua, suba o patch (ex.: `0.6.15` → `0.6.16`); quando o original lançar esse mesmo número, pule para o
seguinte. A tag não pode já existir neste fork.
