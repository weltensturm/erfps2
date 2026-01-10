Texture2D<float4> g_SourceTexture : register(t0);

cbuffer cbPostProcessCommon : register(b4)
{
    float2 g_dynamicScreenPercentage;
    float2 g_texSizeReciprocal;
    float2 g_dynamicScreenPercentage_Primary;
    float2 g_primaryTexSizeReciprocal;
    float2 g_dynamicScreenPercentage_Prev;
    float2 g_prevTexSizeReciprocal;
    float2 g_dynamicScreenPercentage_PrevPrimary;
    float2 g_prevPrimaryTexSizeReciprocal;
};

cbuffer cbToneMap : register(b1)
{
    float3 g_ToneMapInvSceneLumScale;
    int g_ErfpsFlags;
    float4 g_ReinhardParam;
    float4 g_ToneMapParam;
    float4 g_ToneMapSceneLumScale;
    float4 g_AdaptParam;
    float4 g_AdaptCenterWeight;
    float4 g_BrightPassThreshold;
    float4 g_GlareLuminance;
    float4 g_BloomBoostColor;
    float4 g_vBloomFinalColor;
    float4 g_vBloomScaleParam;
    float4x3 g_mtxColorMultiplyer;
    float4 g_vChromaticAberrationRG;
    float4 g_vChromaticAberrationB;
    int4 g_bEnableFlags;
    float4 g_vFeedBackBlurParam;
    float4 g_vVignettingParam;
    float4 g_vHDRDisplayParam;
    float4 g_vChromaticAberrationShapeParam;
    float4 g_vScreenSize;
    float4 g_vSampleDistanceAdjust;
    int4 g_vMaxSampleCount;
    float4 g_vScenePreExposure;
    float4 g_vCameraParam;
};

SamplerState SS_ClampLinear : register(s1);

float4 PSMain(float4 position : SV_Position, float3 coord : TEXCOORD) : SV_TARGET
{
    float2 xy = coord.xy;

    if (g_ErfpsFlags & 1) {
        float2 c = xy - 0.5;
        float2 cScreen = abs(c) * g_vChromaticAberrationShapeParam.xy * g_dynamicScreenPercentage;

        if ((g_ErfpsFlags & 2) && any(cScreen < 0.0014) && all(cScreen < 0.008)) {
            float4 rgba = g_SourceTexture.SampleLevel(SS_ClampLinear, xy, 0);
            return float4((1.0 - rgba.rgb), 1.0);
        }

        float r2 = c.x * c.x;
        float fisheyeStrength = 0.55;

        float f = 1.0 + fisheyeStrength * sqrt(r2) * r2;
        float fMax = 1.0 + fisheyeStrength * 0.125;

        xy = c * f / fMax + 0.5;
    }

    float2 xy2m1 = xy * 2.0 - 1.0;

    float2 chromaR = g_vChromaticAberrationRG.xy;
    float2 chromaG = g_vChromaticAberrationRG.zw;
    float2 chromaB = g_vChromaticAberrationB.xy;

    float2 dynamicScreenPercentage = g_dynamicScreenPercentage;
    float2 texSizeReciprocal = g_texSizeReciprocal;
    float2 texEdge = dynamicScreenPercentage - texSizeReciprocal * 0.5;

    float2 xy2m1ChromaR = xy2m1 * chromaR + xy;
    float2 rCoord = min(xy2m1ChromaR * dynamicScreenPercentage, texEdge);
    float r = g_SourceTexture.SampleLevel(SS_ClampLinear, rCoord, 0).r;

    float2 xy2m1ChromaG = xy2m1 * chromaG + xy;
    float2 gCoord = min(xy2m1ChromaG * dynamicScreenPercentage, texEdge);
    float g = g_SourceTexture.SampleLevel(SS_ClampLinear, gCoord, 0).g;
    
    float2 xy2m1ChromaB = xy2m1 * chromaB + xy;
    float2 bCoord = min(xy2m1ChromaB * dynamicScreenPercentage, texEdge);
    float b = g_SourceTexture.SampleLevel(SS_ClampLinear, bCoord, 0).b;

    return float4(r, g, b, 1.0);
}