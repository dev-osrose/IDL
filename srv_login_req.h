#pragma once


#include "packetfactory.h"
#include <string>

namespace RoseCommon {
namespace Packet {

REGISTER_RECV_PACKET(ePacketType::PAKCS_LOGIN_REQ, SrvLoginReq)
REGISTER_SEND_PACKET(ePacketType::PAKCS_LOGIN_REQ, SrvLoginReq)
class SrvLoginReq : public CRosePacket {
    public:
        SrvLoginReq();
        SrvLoginReq(CRoseReader reader);
        SrvLoginReq(SrvLoginReq&&) = default;
        SrvLoginReq& operator=(SrvLoginReq&&) = default;
        ~SrvLoginReq() = default;
        
        
        struct ChannelInfo : public ISerialize {
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            void set_id(const uint8_t);
            const uint8_t get_id() const;
            void set_lowAge(const uint8_t);
            const uint8_t get_lowAge() const;
            void set_highAge(const uint8_t);
            const uint8_t get_highAge() const;
            void set_capacity(const uint16_t);
            const uint16_t get_capacity() const;
            void set_name(const std::string);
            const std::string get_name() const;
            
            private:
                uint8_t id;
                uint8_t lowAge;
                uint8_t highAge;
                uint16_t capacity;
                std::string name;
        };
        
        struct Test : public ISerialize {
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            void set_a(const uint8_t);
            const uint8_t get_a() const;
            void set_b(const uint8_t);
            const uint8_t get_b() const;
            
            private:
                union {
                    uint8_t a;
                    uint8_t b;
                } data;
        };
        
        
        void set_id(const uint32_t);
        const uint32_t get_id() const;
        void set_channels(const ChannelInfo&);
        const ChannelInfo& get_channels() const;
        
        
        static SrvLoginReq create(const uint32_t&);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
        uint32_t id;
        ChannelInfo channels;
};

}
}
