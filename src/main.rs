use anyhow::Result;
mod chat;
mod llamacpp;
mod openai;
mod term;
mod tools;

const C: &str = r#"
cat > ~/Desktop/raylib_demo/main.c << 'EOF'
#include "raylib.h"

// --- Constants ---
#define PLAYER_WIDTH 50
#define PLAYER_HEIGHT 50
#define PLAYER_SPEED 5.0f
#define BOUNDARY_WIDTH 800
#define BOUNDARY_HEIGHT 450

// --- Game Structures ---
typedef struct {
    Vector2 position;
    Rectangle bounds;
    Color color;
} Player;

// Function to initialize the player
Player InitPlayer(Vector2 startPos) {
    Player p;
    p.position = startPos;
    p.bounds = {startPos.x, startPos.y, PLAYER_WIDTH, PLAYER_HEIGHT};
    p.color = BLUE;
    return p;
}

// Function to update the player position based on input
void UpdatePlayer(Player* player, float deltaTime) {
    // Get current position
    Vector2 newPos = {player->position.x, player->position.y};

    // Movement logic
    if (IsKeyDown(KEY_W)) {
        newPos.y -= PLAYER_SPEED * deltaTime;
    }
    if (IsKeyDown(KEY_S)) {
        newPos.y += PLAYER_SPEED * deltaTime;
    }
    if (IsKeyDown(KEY_A)) {
        newPos.x -= PLAYER_SPEED * deltaTime;
    }
    if (IsKeyDown(KEY_D)) {
        newPos.x += PLAYER_SPEED * deltaTime;
    }

    // Collision Detection (Boundary checking)
    float newX = newPos.x;
    float newY = newPos.y;

    // X-axis bounds
    if (newX < 0) newX = 0;
    if (newX + PLAYER_WIDTH > BOUNDARY_WIDTH) newX = BOUNDARY_WIDTH - PLAYER_WIDTH;

    // Y-axis bounds
    if (newY < 0) newY = 0;
    if (newY + PLAYER_HEIGHT > BOUNDARY_HEIGHT) newY = BOUNDARY_HEIGHT - PLAYER_HEIGHT;

    // Update position only if it's valid
    player->position = (Vector2){newX, newY};
    player->bounds.x = newX;
    player->bounds.y = newY;
}

// Function to draw the entire game scene
void DrawGame(Player player) {
    // 1. Draw the background/boundaries
    DrawRectangle(0, 0, BOUNDARY_WIDTH, BOUNDARY_HEIGHT, DARKGREEN);
    DrawRectangleLines(0, 0, BOUNDARY_WIDTH, BOUNDARY_HEIGHT, BLACK);

    // 2. Draw the moving player object
    DrawRectangleRec(player.bounds, player.color);
    DrawText("Player", player.bounds.x - 30, player.bounds.y - 20, 20, WHITE);

    // 3. Draw FPS counter
    float fps = GetFPS();
    DrawText(TextFormat("FPS: %d", (int)fps), 10, 10, 20, LIME);
}

int main(void)
{
    // Initialization
    const int screenWidth = BOUNDARY_WIDTH + 200; // Extra space for FPS counter
    const int screenHeight = BOUNDARY_HEIGHT + 20;

    InitWindow(screenWidth, screenHeight, "Raylib Demo Project");
    SetTargetFPS(60);

    // Initialize player starting position
    Player player = InitPlayer({50.0f, 50.0f});

    // Main game loop
    while (!WindowShouldClose())
    {
        // --- Update ---
        // Pass a delta time to ensure frame-rate independent movement
        UpdatePlayer(&player, GetFrameTime());

        // --- Draw ---
        BeginDrawing();
            ClearBackground(RAYWHITE);

            // Draw the game world view (clamped to boundary)
            DrawRectangle(0, 0, BOUNDARY_WIDTH, BOUNDARY_HEIGHT, SKYBLUE);
            DrawRectangleLines(0, 0, BOUNDARY_WIDTH, BOUNDARY_HEIGHT, BLACK);

            // Draw the player object
            DrawRectangleRec(player.bounds, BLUE);
            DrawText("Player", player.bounds.x - 30, player.bounds.y - 20, 20, BLACK);

            // Draw a static object/boundary for context
            DrawText("Controls: WASD", 10, BOUNDARY_HEIGHT - 30, 20, GRAY);

            // Draw FPS counter (outside the main game area)
            DrawFPS(screenWidth - 100, 10);

        EndDrawing();
    }

    // De-Initialization
    CloseWindow();
    return 0;
}
EOF
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut terminal = term::Terminal::spawn("zsh", |_| {})?;
    //
    // let e = terminal.execute(C)?;
    //
    // println!("EEEEE = {:#?}", e);
    //
    // return Ok(());

    let terminal = term::Terminal::spawn("zsh", |_| {})?;
    let terminal_tool = tools::TerminalTool::new(terminal, 4096, None);
    let mut tm = tools::ToolManager::new();
    tm.add(terminal_tool);

    let client = openai::OpenaiClient::builder()
        .base_url("http://localhost:8080/v1")
        .build();

    // let llama_client = llamacpp::Client::builder()
    //     .base_url("http://localhost:8080/v1")
    //     .build();
    //
    // let models = llama_client.models().await?;
    //
    // println!("{:?}", models);
    //
    // return Ok(());

    // OmniCoder-Claude-uncensored-V2-Q4_K_M
    // gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M
    // Qwopus3.5-9B-Coder-MTP-Q4_K_M

    let mut builder = openai::models::ChatCompletion::builder()
        .model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M");

    builder = tm.register_all(builder);

    let chat_completion = builder
        .tool_choice(openai::models::ChatCompletionToolChoice::auto())
        .build();

    let c = chat::Chat::new(client, chat_completion, tm);
    let (tx, mut rx) = chat::spawn_chat(c);

    let prompt = r#"
You are working directly on my computer and have full access to the local filesystem and terminal.

Your task is to create a complete, working C project using the Raylib library. Raylib is already installed on the system.

Requirements:

1. First, detect and verify the operating system, distribution, architecture, compiler, and installed Raylib version.
2. Create a new project folder named `raylib_demo` on my Desktop.
3. Inside the project folder, generate all required source files, build scripts, and any supporting files needed for compilation and execution.
4. Implement a simple but visually appealing Raylib demo, for example:

   * a moving player object,
   * keyboard controls,
   * animated objects,
   * collision detection,
   * FPS counter,
   * basic game loop.
5. Write clean, well-structured, and fully commented C code.
6. Create a build script that automatically compiles the project with all required compiler and linker flags for the detected platform.
7. The build script must work without manual modification.
8. Create a run script that:

   * builds the project if needed,
   * launches the executable.
9. Verify that the project compiles successfully.
10. Run the executable and confirm that it starts correctly without errors.
11. If compilation fails, automatically diagnose and fix the issue until the project builds successfully.
12. At the end, provide:

    * the full path to the project folder,
    * the exact build command used,
    * the exact run command used,
    * a brief description of the demo.

Important:

* Do not ask for confirmation.
* Do not stop after generating code.
* Actually create the files and project structure on disk.
* Ensure the final project is fully functional and ready to build and run immediately.
* Prefer portable solutions that work on the detected operating system.
* If multiple compilers are available, choose the most appropriate one automatically.
* Validate all paths before creating files.
* The final result should be a complete, ready-to-run Raylib project located on my Desktop.
        "#;

    let _ = tx.send(Vec::from([
        openai::models::ChatCompletionMessageParam::user(prompt),
    ]));

    while let Some(e) = rx.recv().await {
        println!("{:#?}", e);
    }

    Ok(())
}
