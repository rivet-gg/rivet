import Bullet, { BULLET_CONSTS } from "@shared/bullet";
import ParticleSet, { PARTICLE_TIME } from "@shared/particles";
import { clamp } from "@shared/utils";

export default function drawParticles(ctx: CanvasRenderingContext2D, particleSet: ParticleSet, isSelf: boolean) {
    
    for (const particle of particleSet.parts) {
        ctx.save();

        const opacity = clamp(0, particleSet.timeLeft / PARTICLE_TIME, 1);

        const selfColor = `rgba(0 255 255 / ${opacity})`;
        const otherColor = `rgba(255 0 0 / ${opacity})`;
        
        ctx.strokeStyle = isSelf ? selfColor : otherColor;
        ctx.lineWidth = 4;

        ctx.translate(particle.x, particle.y);
        ctx.rotate(particle.angle);

        ctx.beginPath();
        ctx.moveTo(particle.size / 2, 0);
        ctx.lineTo(-particle.size / 2, 0);
        ctx.stroke();

        ctx.restore();
    }

}
