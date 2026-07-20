package com.versonr7.zavogles;

import android.app.Activity;
import android.os.Bundle;
import android.view.MotionEvent;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.util.Log;

public class ZavoglesActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "ZAVOGLES";

    static {
        System.loadLibrary("k1_app");
    }

    // Native methods
    public static native void nativeOnCreate();
    public static native void nativeOnSurfaceCreated(Surface surface);
    public static native void nativeOnSurfaceChanged(int width, int height);
    public static native void nativeOnSurfaceDestroyed();
    public static native void nativeOnPause();
    public static native void nativeOnResume();
    public static native void nativeOnDestroy();
    public static native void nativeOnTouch(float x, float y, int action);
    public static native void nativeOnFrame();
    public static native void nativeOnRenderThreadExit();

    private SurfaceView surfaceView;
    private boolean running = false;
    private Thread renderThread;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.i(TAG, "Activity onCreate");
        
        surfaceView = new SurfaceView(this);
        surfaceView.getHolder().addCallback(this);
        setContentView(surfaceView);
        
        nativeOnCreate();
    }

    @Override
    protected void onPause() {
        super.onPause();
        Log.i(TAG, "Activity onPause");
        running = false;
        if (renderThread != null) {
            try {
                renderThread.join(100);
            } catch (InterruptedException e) {
                Log.e(TAG, "join interrupted", e);
            }
        }
        nativeOnPause();
    }

    @Override
    protected void onResume() {
        super.onResume();
        nativeOnResume();
        if (surfaceView.getHolder().getSurface() != null && surfaceView.getHolder().getSurface().isValid()) {
            if (renderThread == null || !renderThread.isAlive()) {
                running = true;
                renderThread = new Thread(this::renderLoop);
                renderThread.start();
            }
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        Log.i(TAG, "Activity onDestroy");
        nativeOnDestroy();
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        int action = event.getActionMasked();
        float x = event.getX();
        float y = event.getY();
        nativeOnTouch(x, y, action);
        return true;
    }

    // SurfaceHolder.Callback
    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.i(TAG, "Surface created");
        nativeOnSurfaceCreated(holder.getSurface());
        running = true;
        renderThread = new Thread(this::renderLoop);
        renderThread.start();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        Log.i(TAG, "Surface changed: " + width + "x" + height);
        nativeOnSurfaceChanged(width, height);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.i(TAG, "Surface destroyed");
        running = false;
        try {
            renderThread.join();
        } catch (InterruptedException e) {
            Log.e(TAG, "Render thread interrupted", e);
        }
        // لا تستدع nativeOnSurfaceDestroyed هنا
    }

    private void renderLoop() {
        while (running) {
            nativeOnFrame();
            try {
                Thread.sleep(16);
            } catch (InterruptedException e) {
                break;
            }
        }
        // بعد الخروج من الحلقة، نظّف موارد GL على نفس الخيط
        nativeOnRenderThreadExit();
    }
}
