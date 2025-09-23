# 🚚 Colis Privé Testing Tool

Herramienta simple para probar directamente la API de Colis Privé.

## 🚀 Uso

```bash
cd backend/testing-tool
cargo run
```

## 📋 Funcionalidades

1. **🔐 Autenticación**: Pide credenciales y obtiene token de Colis Privé
2. **📦 Obtener Paquetes**: Prueba la obtención de paquetes con el token

## 🔧 Características

- **Interfaz simple**: Solo terminal, sin dependencias complejas
- **Respuestas completas**: Muestra todos los headers y body sin filtros
- **Debugging fácil**: Puedes copiar/pegar las respuestas para investigar
- **Comandos curl reales**: Usa exactamente los mismos comandos que el backend

## 📝 Flujo

1. Ejecutas `cargo run`
2. Ingresas tus credenciales (username, password, matrícula, sociedad)
3. El tool se autentica y guarda el token
4. Seleccionas "Probar obtener paquetes"
5. Ves toda la respuesta completa para debugging

## 🎯 Propósito

Esta herramienta te permite:
- Probar credenciales directamente con Colis Privé
- Ver exactamente qué devuelve la API
- Debuggear problemas de autenticación o obtención de paquetes
- Copiar/pegar respuestas para análisis