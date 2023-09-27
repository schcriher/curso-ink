# AnsehenDAO:

**AnsehenDAO** (_"ánsiindao"_) es una plataforma de gestión de reputación que busca premiar las contribuciones que sus miembros realizan a la organización. Los fondos se reparten a los miembros en base a su reputación en cada ronda de reparto.

El nombre **AnsehenDAO** viene de la unión de **Ansehen** y **DAO**, donde **Ansehen** significa "reputación" en alemán y **DAO**, del inglés Decentralized Autonomous Organization, significa "organización autónoma descentralizada".

## Funcionamiento

Los miembros "contribuyentes" aportan distintos tipos de acciones off-chain a la organización, ya sea trabajo físico o intelectual; mientras que la organización, a través de sus miembros "administradores", realizan distintas rondas de votación con fondos que la organización consigue para repartir (on-chain) entre sus contribuyentes en base a la reputación de cada contribuyente en cada ronda, esta reputación es calculada a partir de los votos que cada contribuyente realiza en las rondas. Los contribuyentes con mayor reputación tendrán a su vez mayor poder de voto. Al finalizar cada ronda se entregan certificados NFT a los 3 contribuyentes con mayor reputación y se reinicializan los valores de reputación y cantidad de votos emitidos de cada contribuyente.

## Estructura

### Consideraciones de diseño

- Se emiten tres tipos de eventos, uno cuando se crea una ronda de votación (`NewRound`), uno cuando se cierra la ronda de votación (`CloseRound`) y otro cuando se emite un voto (`VoteCast`).
- Los votos se pueden emitir pero no eliminar, para evitar que una vez emitidos los votos, algún contribuyente elimine los suyos y con mayor reputación cambie su votación.
- El `storage` del contrato tiene los siguientes campos:

  - `rounds`: es un mapping que almacena todas las rondas creadas, recuperables con su ID que es un número creciente, comenzando por 1 siendo la última ronda creada la almacenada en el campo `current_round_id`.
  - `current_round_id`: ID de la última ronda creada. Se considera un sistema con una sola ronda activa a la vez

  - `min_elapsed_milliseconds`: tiempo mínimo para que una ronda quede abierta.

  - `members`: un mapping que almacena los id de todos los miembros y su rol (`admin`, `contributor`).

  - `contributors`: un mapping que almacena los id de todos los contribuyentes y su información actual, la cual consta de la reputación y los votos emitidos en la ronda actual.

  - `contributors_list`: lista con todos los id de los contribuyentes para poder iterar sobre ellos al momento de hacer la distribución de los fondos y el reseteo de los valores de reputación y votos emitidos. Este campo fue marcado como `Lazy` para evitar que se cargue con cada llamada al contrato, ya que es un vector.

  - `nft_ref`: una referencia al contrato de los NFT, recompensa para los tres contribuyentes con mayor reputación.

- El cálculo de la reputación de un contribuyente se realiza con la ecuación propuesta en el enunciado. Con la modificación de que el cálculo de la raíz cuadrada de la reputación del que emite el voto se realiza con una fórmula rápida que da un resultado aproximado (archivo `tools.rs`).

- Se pueden agregar mas de un administrador a la organización, sin embargo al eliminarlos el que elimina no puede auto-eliminarse para evitar que se quede sin administradores la organización.

- Se pueden agregar y eliminar contribuyentes a la organización, sin embargo no debe estar activa una ronda para evitar manipulaciones mientras se vota.

- Una ronda puede ser abierta solo si no hay una ya abierta no finalizada y el contrato tiene suficientes fondos (según el valor pasado por parámetro) y que quede al menos la cantidad mínima de existencia.

- Al cerrar una ronda se calculan las cantidades que le corresponde a cada contribuyente, se resetean su valores y luego se envían los NFT a los 3 contribuyentes con mayor reputación, esta se realiza ordenando una lista temporal que se crea y se van sacando del final (`.pop()`) en caso de no haber contribuyentes simplemente no se envían los NFTs.

- Se tienen métodos de consulta para saber el tiempo mínimo para una ronda, la dirección del contrato para hacer aportes y el tiempo (timestamp) actual.

- Por otro lado se implementa el trait `VoteTrait` el cual permite emitir un voto y consultar la reputación.

- Todos los mensajes (transacciones) devuelven un `Result` con el valor correspondiente o nada, o un error de los definidos en el archivo de errores, en principio no debería generar ningún panic.

### Mejoras futuras

- Las pruebas de integración deberían ampliarse para cubrir todos los casos válidos y todos los casos de error, ya se que comprueban muchas situaciones.

- Los NFT podrían almacenar mas información sobre la ronda, reputación, cantidad de votantes, etc.

- Se podría crear una UI para interactuar con la organización de forma mas sencilla, ya que es uno de los mayores obstáculos en el mundo de la blockchain.

---

## Instalación del entorno de derarrollo (GNU/Linux)

### Rust & Cargo

- Web: https://www.rust-lang.org/tools/install
- Acción: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Ink!

- Webs:
  - https://use.ink/getting-started/setup/
  - https://github.com/paritytech/cargo-contract
- Acciones:
  - `rustup install 1.72`
  - `rustup default 1.72`
  - `rustup component add rust-src --toolchain 1.72`
  - `rustup component add clippy --toolchain 1.72`
  - `rustup target add wasm32-unknown-unknown --toolchain 1.72`
  - `cargo install --force --version 3.2.0 cargo-contract`
  - `cargo check`

### Blockchain Node

- Web: https://github.com/paritytech/substrate-contracts-node/releases
- Acciones:
  - Descargar `substrate-contracts-node-linux.tar.gz`
  - Colocar `substrate-contracts-node` en una carpeta del PATH del sistema

### Resumen

| Software                 | Versión                                |
| ------------------------ | -------------------------------------- |
| OS                       | Debian Testing (13 "trixie")           |
| rustup                   | 1.26.0 (5af9b9484 2023-04-05)          |
| rustc                    | 1.72.0 (5680fa18f 2023-08-23)          |
| cargo                    | 1.72.0 (103a7ff2e 2023-08-15)          |
| cargo-contract           | 3.2.0-unknown-x86_64-unknown-linux-gnu |
| substrate-contracts-node | 0.31.0-c8863fe08b7                     |

---

## Acciones de desarrollo

### Ejecución de pruebas

```Bash
cargo test --lib --features e2e-tests -- --nocapture
```

### Compilación

```Bash
cargo contract build --target wasm --manifest-path contracts/organization/Cargo.toml
# Resultado en: target/ink/nft/nft.contract

cargo contract build --target wasm --manifest-path contracts/nft/Cargo.toml
# Resultado en: target/ink/organization/organization.contract
```

### Ejecución local

- Ejecutar `substrate-contracts-node`
- Webs:
  - https://contracts-ui.substrate.io
  - https://polkadot.js.org/apps/

---

## Avances del proyecto

**Objetivo:** Armar una organización que premie a sus contribuyentes según su reputación.

<br/>

### 📝 Clase 1

- [x] Configurar el entorno de desarrollo local
- [x] Generar un contrato flipper
- [x] Generar un repositorio git personal para el seguimiento del trabajo práctico
- [x] Subir el código del contrato generado
- [x] Compartir el repositorio en el canal de discord para el trackeo del mismo

> ```Bash
> cargo contract new flipper
> cd flipper
>
> cargo test --package flipper --lib -- flipper::tests --nocapture
> cargo contract build --target wasm
>
> git init
> git add .
> git commit -m "class #1"
> git branch -M master
> git remote add origin git@github.com:schcriher/curso-ink.git
> git push -u origin master
> ```

<br/>

### 📝 Clase 2

Modificar el smart contract para empezar a darle forma a nuestra organización:

**Storage:**

- [x] Incluir a los contribuyentes con su reputación asociada (usar vectores).
- [x] Incluir una cuenta administradora, que podrá agregar/eliminar contribuyentes.

**Mensajes:**

- [x] Agregar/Eliminar contribuyente
- [x] Votar (sólamente un contribuyente puede votar a otro)
- [x] Consultar reputación de contribuyente

<br/>

> **Notas:** _para esta etapa del desarrollo se asumen las siguientes condiciones:_
>
> - _La "reputación" es la suma de votos que tiene un contribuyente_
> - _Un contribuyente puede votar infinitamente a otros contribuyentes_

<br/>

### 📝 Clase 3

- [x] Modificar el storage para utilizar Mappings en lugar de Vectores
- [x] Modificar lógica para que el poder de voto se corresponda con la reputación del contribuyente
      (mayor reputación → mayor poder de voto)
- [x] Emitir un evento por cada voto
- [x] Agregar los siguientes controles:
  - El único que puede agregar o eliminar contribuyentes es el Admin
  - Los únicos que pueden votar son los contribuyentes registrados
  - La reputación es privada. Cada contribuyente puede consultar únicamente la propia

<br/>

> **Notas:** _Proyecto renombrado a "Organization", además se asumen las siguientes condiciones:_
>
> - _Hasta tener una definición mas completa de la lógica de negocio:_
>
>   - _Se mantiene la emisión ilimitada de votos_
>   - _Se mantiene la emisión de votos positivos unicamente_
>   - _El poder de voto se divide en dos categorías:_
>
>     - _contribuyentes con menos de 10 votos: suman 1 a la reputación con cada voto_
>     - _contribuyentes con 10 o mas votos: suman 2 a la reputación con cada voto_
>
> - _El evento emitido en la votación contiene las direcciones del votante y el votado_

<br/>

### 📝 Clase 4

- [x] Crear un contrato PSP34 (Utilizar Templates de OpenBrush) que sirva de certificado de votación
- [x] Transferir al contribuyente un NFT que certifique su voto
- [x] Definir un trait que represente el comportamiento de votación e implementarlo en el contrato:
  - Votar
  - Obtener reputación/votos de un contribuyente

<br/>

> **Problema de dependencias:**
>
> - Conflicto:
>   - `subxt-codegen v0.31.0` (paquete no manejado directamente) depende de `rustc >= 1.70`
>   - `cargo-contract v3.2.0` (paquete nuevo) soporta `rustc > 1.69`
> - Acciones:
>   - Pasos de la sección de instalacion de [Ink!](#ink) (actualizado)
>
> **Notas:**
>
> - Al interactuar con un segundo contrato ya no es posible realizar pruebas unitarias con datos falsos como se estaba haciendo hasta ahora. El archivo `ink_env-4.3.0/src/engine/off_chain/impls.rs` en su función `instantiate_contract` (línea 488) lo define como "no implementado" con el mensaje _"off-chain environment does not support contract instantiation"_, por lo tanto se eliminan todas las pruebas unitarias (quedan en el historial de git).

<br/>

### 📝 Clase 5

_Clase teórica sobre Chain Extensions, sin cambios en el proyecto._

<br/>

### 📝 Clase 6

- [x] Agregar tests de integración y E2E al código de la clase #4. No hace falta que cubran el 100% del código.

<br/>

> **Notas:**
>
> - Se agregó una prueba unitaria, no se agregaron pruebas de integración debido a que el contrato principal instancia un segundo contrato y para probar esto es necesario hacerlo on-chain, y por úlitmo se agregaron pruebas end-2-end que testean las cuatro funciones para casos correctos, en las cuales se hizo uso de macros para disminuir la cantidad de líneas de código.
> - El testeo de casos de error se deja para la entrega final.

<br/>

### 📝 Clase 7

**Enunciado final:** El objetivo del trabajo práctico es crear una **plataforma de gestión de reputación** según las contribuciones realizadas a una organización.

**Reglas:**

- **Organización**

  - Una **organización** tiene miembros.
  - Los miembros tienen roles: **Admin** o **Contributor**.
  - Los **contributors** participan haciendo **aportes off-chain**.
  - Los aportes off-chain se **valorizarán** mediante votos on-chain entre contributors.
  - La organización, **mediante su Admin**, abrirá **rondas de votación con una duración determinada que podrá variar entre las diferentes rondas**.
  - Al momento de crear la ronda de votación, el Admin deberá indicar:
    - **el monto de fondos a repartir** entre los contributors.
    - **la cantidad de votos** que podrá efectuar cada uno de ellos.
  - **Los fondos deberán ser cargados por el Admin.**

- **Votación**

  - Los contributors podrán votar de forma **positiva** o **negativa** a otros contributors.
  - Estos votos **impactarán en el valor de reputación** del contributor votado.
  - **El valor de reputación de un contributor nunca podrá ser menor a 1**.
  - El poder de voto de cada contributor será **proporcional a su valor de reputación**. _La fórmula quedará a criterio de cada uno_.
    - Ejemplo: f(member_pts, target_pts, value) = target_pts + value \* sqrroot(member_pts)
    - Value = +PTS o -PTS

- **Premiación**
  - Al finalizar la ronda de votación, **los fondos se repartirán entre los contributors en base a su valor de reputación** a partir de una transacción ejecutada por el Admin.
  - Luego de que se repartan los fondos, **los valores de reputación se resetearán**.
  - Se **entregarán badges (NFTs)** a los 3 contributors con mayor valor de reputación (Gold, Silver and Bronze).

**Entregable:**

- Se deberá presentar un repositorio de código con los contratos.
- El README del repo deberá contener la explicación de la solución.
