# Netero

Un asistente de línea de comandos para modelos de lenguaje escrito en Rust, diseñado para flujos de trabajo centrados en la terminal.

**También disponible en inglés:** [See README in English](../../README.md)

## Estado del proyecto

Netero es un software experimental. Las características están incompletas y están sujetas a cambios.

## Variables de entorno

Netero requiere variables de entorno específicas del proveedor para ser configuradas.

* `CODE_API_KEY`
  Clave API utilizada para el proveedor `codestral`.

* `OPENROUTER_API_KEY`
  Clave API utilizada para el proveedor `openrouter`.

El proveedor `ollama` puede ser utilizado **localmente** sin una clave API.

Por el momento, estas son las únicas opciones de configuración soportadas. El manejo de proveedores se espera que se vuelva más flexible en el futuro.

## Uso

Interfaz de línea de comandos para interactuar con modelos de lenguaje.

```
Uso: netero [OPCIONES] [PROMPT] [COMANDO]
```

Si se proporciona entrada a través de `stdin`, se utilizará como contexto adicional para el prompt.

### Comandos

* `chat`
  Abre una sesión de chat minimalista

* `commit`
  Genera un mensaje de commit a partir de los cambios en espera

* `prompt`
  Envía un prompt al modelo de lenguaje e imprime la respuesta

### Argumentos

* `[PROMPT]`
  Prompt pasado al modelo de lenguaje

### Opciones

* `-p, --provider <PROVIDER>`
  Proveedor de modelos de lenguaje (por defecto: `codestral`)

* `-v, --verbose`
  Habilita la salida detallada

* `-h, --help`
  Muestra la ayuda

* `-V, --version`
  Muestra la versión

## Ejemplos

### Prompt básico

```sh
netero "Explica la diferencia entre enlaces duros y simbólicos"
```

### Usando stdin para prompts más largos

```sh
cat README.md | netero "Resume el README del proyecto"
```

### Generar un mensaje de commit de Git

```sh
netero commit | git commit -F - --edit
```

### Usando un proveedor diferente

```sh
netero -p openrouter "Explica cómo systemd gestiona los servicios"
```

### Salida detallada

```sh
netero -v "Explica el modelo de propiedad de Rust"
```

### Procesar una página de manual

```sh
man tmux | netero "¿Cómo puedo dividir una ventana de tmux?"
```

### Analizar la salida de un comando

```sh
ps aux | netero "¿Qué procesos están consumiendo más recursos?"
```

### Enviar la salida a otro comando
```sh
ss -tulpen | netero "Resume los sockets de escucha activos" | mdless
```

## Licencia

BSD 2-Clause
